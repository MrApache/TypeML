#use <schema.tmd>
//#import <schema::*> //Import all types from namespace "schema"

$expr ColorComponent -> base::Component {
    target:  "Player",
    path:    "color",
    with:    ["ABC", "123", "ABC"],
    without: ["ABC", "123", "ABC"],
}

$struct NodeBorder -> UiRect {
    left: 10px,
    right: 10px,
    top: 10px,
    bottom: 10px,
}

<base::Layout@Root>
    <Node width=100% height=100% border=$NodeBorder/>
    <BackgroundColor self=$ColorComponent/>
    <base::ItemTemplate>
        <base::Entity>
            <Node width=30px height=40px border={{
                left: 2px,
                right: 2px,
                top: 3px,
                bottom: 6px,
            }}/>
            <BackgroundColor self="#FFFFFF"/>
            <Counter default_value=100 triggers=[10, 100, 1000]/>
        </base::Entity>
    </base::ItemTemplate>
</base::Layout>

//TODO Error locations
//TODO Empty list error
//TODO Annotations errors
//TODO A namespace identifier should be equal file name (extend fix)
//TODO Directive 'Include' that allow include type from namespace (+ Alias support)
//TODO Item attachment
//TODO Fix duplicate fields in definitions
//TODO Fix duplicate arguments in elements
//TODO Fix duplicate struct fields
//TODO Multiple configs
//TODO Metadata assignment
//TODO Expression restrict support
