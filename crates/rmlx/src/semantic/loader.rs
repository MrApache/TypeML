use reqwest::StatusCode;
use std::{ffi::OsStr, path::Path};
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("local file not found or not a file: {0}")]
    LocalNotFound(String),
    #[error("file does not have .rmlx extension")]
    WrongExtension,
    #[error("http error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("http status not successful: {0}")]
    HttpStatus(reqwest::StatusCode),
    #[error("failed to read local file: {0}")]
    LocalReadError(#[from] std::io::Error),
    #[error("could not determine filename/extension for remote resource")]
    CannotDetermineRemoteFilename,
}

/// Загружает содержимое источника `source` как String, если:
///  - источник доступен (локально или по http(s))
///  - файл имеет расширение .rmlx (проверяется по пути или Content-Disposition)
///
/// Поддерживается:
///  - "http://..." и "https://..."
///  - "file:///abs/path/to/file.rmlx"
///  - "/local/path/to/file.rmlx" или "relative/path.rmlx"
pub fn load_rmlx(url: &Url) -> Result<String, LoadError> {
    match url.scheme() {
        //"http" | "https" => load_remote_rmlx(url).await,
        "file" => match url.to_file_path() {
            Ok(path_buf) => load_local_path(&path_buf),
            Err(()) => Err(LoadError::InvalidUrl(url.to_string())),
        },
        _ => {
            // трактуем как локальный путь (например, "C:\..." на Windows без схемы невалиден)
            let path = Path::new(url.as_str());
            load_local_path(path)
        }
    }
}

fn load_local_path(path: &Path) -> Result<String, LoadError> {
    if !path.exists() || !path.is_file() {
        return Err(LoadError::LocalNotFound(path.display().to_string()));
    }
    // Проверяем расширение .rmlx (регистронезависимо)
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("rmlx") => {
            // читаем файл (асинхронно)
            //let s = tokio::fs::read_to_string(path).await?;
            let s = std::fs::read_to_string(path)?;
            Ok(s)
        }
        _ => Err(LoadError::WrongExtension),
    }
}

async fn load_remote_rmlx(url: &Url) -> Result<String, LoadError> {
    //let url = Url::parse(url_str).map_err(|_| LoadError::InvalidUrl(url_str.to_string()))?;
    // Попробуем выяснить расширение из пути URL
    if has_rmlx_extension_in_url_path(url) {
        // Проведём сначала HEAD, чтобы убедиться, что ресурс доступен
        let client = reqwest::Client::new();
        let head_resp = client.head(url.clone()).send().await;

        match head_resp {
            Ok(resp) => {
                if !resp.status().is_success() {
                    // Если HEAD вернул неуспех, можно попытаться GET (некоторые сервера не поддерживают HEAD)
                    if resp.status() != StatusCode::METHOD_NOT_ALLOWED {
                        return Err(LoadError::HttpStatus(resp.status()));
                    }
                }
            }
            Err(e) => {
                // если HEAD упал с ошибкой сети — попробуем GET и позволим ошибку прокинуться
                let _ = e;
            }
        }

        // GET контента
        let body = reqwest::get(url.clone()).await?;
        if !body.status().is_success() {
            return Err(LoadError::HttpStatus(body.status()));
        }
        let text = body.text().await?;
        Ok(text)
    } else {
        // Если в URL нет расширения, попробуем HEAD/GET и посмотреть headers -> Content-Disposition
        let client = reqwest::Client::new();
        let head = client.head(url.clone()).send().await;

        let mut filename_opt: Option<String> = None;

        if let Ok(resp) = head {
            if resp.status().is_success() {
                if let Some(cd) = resp.headers().get(reqwest::header::CONTENT_DISPOSITION) {
                    if let Ok(cd_val) = cd.to_str() {
                        if let Some(name) = extract_filename_from_content_disposition(cd_val) {
                            filename_opt = Some(name);
                        }
                    }
                }
            } else if resp.status() != StatusCode::METHOD_NOT_ALLOWED {
                return Err(LoadError::HttpStatus(resp.status()));
            }
        }

        // если всё ещё не ясно — делаем GET и снова смотрим headers (и при удаче читаем тело)
        let resp = client.get(url.clone()).send().await?;
        if !resp.status().is_success() {
            return Err(LoadError::HttpStatus(resp.status()));
        }

        // проверить Content-Disposition в ответе
        if filename_opt.is_none() {
            if let Some(cd) = resp.headers().get(reqwest::header::CONTENT_DISPOSITION) {
                if let Ok(cd_val) = cd.to_str() {
                    if let Some(name) = extract_filename_from_content_disposition(cd_val) {
                        filename_opt = Some(name);
                    }
                }
            }
        }

        // если filename найден и имеет расширение .rmlx — ок
        if let Some(fname) = filename_opt {
            if Path::new(&fname)
                .extension()
                .and_then(OsStr::to_str)
                .is_some_and(|s| s.eq_ignore_ascii_case("rmlx"))
            {
                // получили тело как текст
                let text = resp.text().await?;
                return Ok(text);
            }

            return Err(LoadError::WrongExtension);
        }

        // если ни в URL ни в Content-Disposition нет расширения — откажемся
        Err(LoadError::CannotDetermineRemoteFilename)
    }
}

fn has_rmlx_extension_in_url_path(url: &Url) -> bool {
    // берем путь, смотрим extension
    let path = url.path(); // e.g. "/dir/file.rmlx"
    Path::new(path)
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|s| s.eq_ignore_ascii_case("rmlx"))
}

/// Простая вычитка имени файла из Content-Disposition
fn extract_filename_from_content_disposition(cd: &str) -> Option<String> {
    // ищем filename=... (учитываем кавычки)
    // примеры: attachment; filename="example.rmlx" или attachment; filename=example.rmlx
    let lower = cd.to_lowercase();
    if let Some(pos) = lower.find("filename=") {
        // найдём с оригинальной строки от pos+len("filename=")
        let start = pos + "filename=".len();
        let tail = &cd[start..];
        // убрать пробелы
        let tail = tail.trim_start();
        if let Some(rest) = tail.strip_prefix('"') {
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        } else {
            let end = tail.find(';').unwrap_or(tail.len());
            return Some(tail[..end].trim().to_string());
        }
    }
    None
}
