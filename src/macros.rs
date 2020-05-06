// blatantly inspired by serde_json::json

#[macro_export]
macro_rules! ron {
    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: __ron!($($ron)+)
    //////////////////////////////////////////////////////////////////////////
    (true) => {
        $crate::Value::Bool(true)
    };
    (false) => {
        $crate::Value::Bool(false)
    };

    ([]) => {
        $crate::Value::Array($crate::std::vec![])
    };
    ([ $($tt:tt)+ ]) => {
        $crate::Value::Array($crate::__ron!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::Value::Map($crate::value::Map::new())
    };
    ({ $($tt:tt)+ }) => {
        $crate::Value::Map($crate::__ron!(@map [] [] $($tt)+).into_iter().collect())
    };

    (()) => {
        $crate::Value::Struct($crate::value::Struct {
            name: None,
            fields: None,
        })
    };
    ($Name:ident) => {
        $crate::Value::Struct($crate::value::Struct {
            name: Some($crate::std::stringify!($Name)),
            fields: None,
        })
    };
    (( $($tt:tt)+ )) => {
        $crate::Value::Struct($crate::value::Struct {
            name: None,
            fields: Some(Box::new($crate::__ron!(@fields [] $($tt)+).into_iter().collect())),
        })
    };
    ($Name:ident ()) => {
        $crate::Value::Struct($crate::value::Struct {
            name: Some($crate::std::stringify!($Name)),
            fields: Some(Box::new($crate::value::Fields::Unnamed($crate::std::vec![]))),
        })
    };
    ($Name:ident ( $($tt:tt)+ )) => {
        $crate::Value::Struct($crate::value::Struct {
            name: Some($crate::std::stringify!($Name)),
            fields: Some(Box::new($crate::__ron!(@fields [] $($tt)+).into_iter().collect())),
        })
    };

    // Any Serialize type: numbers, strings, (borrowed) variables, etc.
    // Must be after every other rule.
    ($other:expr) => {
        $crate::to_value(&$other).unwrap()
    };
}

// future: add some error handling branches like in json! if they're helpful

#[doc(hidden)]
#[macro_export]
macro_rules! __ron {
    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an array [...].
    // Produces a vec![...] of the elements.
    //
    // Must be invoked as: __ron!(@array [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////
    (@array [$($done:tt)*]) => {
        $crate::std::vec![$($done)*]
    };

    (@array [$($done:tt)*] [$($inner:tt)*] $(, $($rest:tt)*)?) => {
        $crate::__ron!(@array
            [$($done)* $crate::ron!([$($inner)*]),]
            $($($rest)*)?)
    };
    (@array [$($done:tt)*] {$($inner:tt)*} $(, $($rest:tt)*)?) => {
        $crate::__ron!(@array
            [$($done)* $crate::ron!({$($inner)*}),]
            $($($rest)*)?)
    };
    (@array [$($done:tt)*] ($($inner:tt)*) $(, $($rest:tt)*)?) => {
        $crate::__ron!(@array
            [$($done)* $crate::ron!(($($inner)*)),]
            $($($rest)*)?)
    };

    (@array [$($done:tt)*] $name:ident ($($inner:tt)*) $(, $($rest:tt)*)?) => {
        $crate::__ron!(@array
            [$($done)* $crate::ron!($name ($($inner)*)),]
            $($($rest)*)?)
    };
    (@array [$($done:tt)*] $name:ident $(, $($rest:tt)*)?) => {
        $crate::__ron!(@array
            [$($done)* $crate::ron!($name),]
            $($($rest)*)?)
    };
    (@array [$($done:tt)*] $expr:expr $(, $($rest:tt)*)?) => {
        $crate::__ron!(@array
            [$($done)* $crate::ron!($expr),]
            $($($rest)*)?)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an map {...}.
    // Produces a vec![...] of the elements.
    //
    // Must be invoked as: __ron!(@map [] [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////
    (@map [$($done:tt)*] []) => {
        $crate::std::vec![$($done)*]
    };
    (@map [$($done:tt)*] [$key:expr]) => {
        // better error message: "unexpected end of macro invocation"
        $crate::__ron!()
    };

    (@map [$($done:tt)*] [] [$($inner:tt)*] : $($rest:tt)*) => {
        $crate::__ron!(@map
            [$($done)*]
            [$crate::ron!([$($inner)*])]
            $($rest)*)
    };
    (@map [$($done:tt)*] [] {$($inner:tt)*} : $($rest:tt)*) => {
        $crate::__ron!(@map
            [$($done)*]
            [$crate::ron!({$($inner)*})]
            $($rest)*)
    };
    (@map [$($done:tt)*] [] ($($inner:tt)*) : $($rest:tt)*) => {
        $crate::__ron!(@map
            [$($done)*]
            [$crate::ron!(($($inner)*))]
            $($rest)*)
    };

    (@map [$($done:tt)*] [] $name:ident ($($inner:tt)*) : $($rest:tt)*) => {
        $crate::__ron!(@map
            [$($done)*]
            [$crate::ron!($name ($($inner)*))]
            $($rest)*)
    };
    (@map [$($done:tt)*] [] $name:ident : $($rest:tt)*) => {
        $crate::__ron!(@map
            [$($done)*]
            [$crate::ron!($name)]
            $($rest)*)
    };
    (@map [$($done:tt)*] [] $expr:literal : $($rest:tt)*) => {
        $crate::__ron!(@map
            [$($done)*]
            [$crate::ron!($expr)]
            $($rest)*)
    };
    (@map [$($done:tt)*] [] $expr:expr => $($rest:tt)*) => {
        $crate::__ron!(@map
            [$($done)*]
            [$crate::ron!($expr)]
            $($rest)*)
    };

    (@map [$($done:tt)*] [$key:expr] [$($inner:tt)*] $(, $($rest:tt)*)?) => {
        $crate::__ron!(@map
            [$($done)* ($key, $crate::ron!([$($inner)*])),]
            []
            $($($rest)*)?)
    };
    (@map [$($done:tt)*] [$key:expr] {$($inner:tt)*} $(, $($rest:tt)*)?) => {
        $crate::__ron!(@map
            [$($done)* ($key, $crate::ron!({$($inner)*})),]
            []
            $($($rest)*)?)
    };
    (@map [$($done:tt)*] [$key:expr] ($($inner:tt)*) $(, $($rest:tt)*)?) => {
        $crate::__ron!(@map
            [$($done)* ($key, $crate::ron!(($($inner)*))),]
            []
            $($($rest)*)?)
    };

    (@map [$($done:tt)*] [$key:expr] $name:ident ($($inner:tt)*) $(, $($rest:tt)*)?) => {
        $crate::__ron!(@map
            [$($done)* ($key, $crate::ron!($name ($($inner)*))),]
            []
            $($($rest)*)?)
    };
    (@map [$($done:tt)*] [$key:expr] $name:ident $(, $($rest:tt)*)?) => {
        $crate::__ron!(@map
            [$($done)* ($key, $crate::ron!($name)),]
            []
            $($($rest)*)?)
    };
    (@map [$($done:tt)*] [$key:expr] $expr:expr $(, $($rest:tt)*)?) => {
        $crate::__ron!(@map
            [$($done)* ($key, $crate::ron!($expr)),]
            []
            $($($rest)*)?)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of a struct (...).
    // Produces a vec![...] of the elements.
    //
    // Must be invoked as: __ron!(@fields [] $($tt)*)
    //////////////////////////////////////////////////////////////////////////
    (@fields $(@$mode:ident)? [$($done:tt)*]) => {
        $crate::std::vec![$($done)*]
    };

    (@fields $(@named)? [$($done:tt)*] $field:ident : [$($inner:tt)*] $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @named
            [$($done)* ($crate::std::stringify!($field), $crate::ron!([$($inner)*])),]
            $($($rest)*)?)
    };
    (@fields $(@named)? [$($done:tt)*] $field:ident : {$($inner:tt)*} $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @named
            [$($done)* ($crate::std::stringify!($field), $crate::ron!({$($inner)*})),]
            $($($rest)*)?)
    };
    (@fields $(@named)? [$($done:tt)*] $field:ident : ($($inner:tt)*) $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @named
            [$($done)* ($crate::std::stringify!($field), $crate::ron!(($($inner)*))),]
            $($($rest)*)?)
    };

    (@fields $(@named)? [$($done:tt)*] $field:ident : $name:ident ($($inner:tt)*) $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @named
            [$($done)* ($crate::std::stringify!($field), $crate::ron!($name ($($inner)*))),]
            $($($rest)*)?)
    };
    (@fields $(@named)? [$($done:tt)*] $field:ident : $name:ident $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @named
            [$($done)* ($crate::std::stringify!($field), $crate::ron!($name)),]
            $($($rest)*)?)
    };
    (@fields $(@named)? [$($done:tt)*] $field:ident : $expr:expr $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @named
            [$($done)* ($crate::std::stringify!($field), $crate::ron!($expr)),]
            $($($rest)*)?)
    };

    (@fields $(@unnamed)? [$($done:tt)*] [$($inner:tt)*] $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @unnamed
            [$($done)* ($crate::ron!([$($inner)*])),]
            $($($rest)*)?)
    };
    (@fields $(@unnamed)? [$($done:tt)*] {$($inner:tt)*} $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @unnamed
            [$($done)* ($crate::ron!({$($inner)*})),]
            $($($rest)*)?)
    };
    (@fields $(@unnamed)? [$($done:tt)*] ($($inner:tt)*) $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @unnamed
            [$($done)* ($crate::ron!(($($inner)*))),]
            $($($rest)*)?)
    };

    (@fields $(@unnamed)? [$($done:tt)*] $name:ident ($($inner:tt)*) $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @unnamed
            [$($done)* ($crate::ron!($name ($($inner)*))),]
            $($($rest)*)?)
    };
    (@fields $(@unnamed)? [$($done:tt)*] $name:ident $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @unnamed
            [$($done)* ($crate::ron!($name)),]
            $($($rest)*)?)
    };
    (@fields $(@unnamed)? [$($done:tt)*] $expr:expr $(, $($rest:tt)*)?) => {
        $crate::__ron!(@fields @unnamed
            [$($done)* ($crate::ron!($expr)),]
            $($($rest)*)?)
    };
}

#[test]
fn macro_works_on_simple_example() {
    let ron = ron! {
        GameConfig( // optional struct name
            window_size: (800, 600),
            window_title: "PAC-MAN",
            fullscreen: false,

            mouse_sensitivity: 1.4,
            key_bindings: {
                "up": Up,
                "down": Down,
                "left": Left,
                "right": Right,

                // Uncomment to enable WASD controls
                /*
                "W": Up,
                "A": Down,
                "S": Left,
                "D": Right,
                */
            },

            difficulty_options: (
                start_difficulty: Easy,
                adaptive: false,
            ),
        )
    };
    dbg!(&ron);
    println!("{}", crate::to_string(&ron).unwrap());
}
