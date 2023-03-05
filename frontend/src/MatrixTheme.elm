module MatrixTheme exposing (..)

import Element exposing (Element, html, rgb255)
import Element.Font as Font
import Svg
import Svg.Attributes


yellowColor =
    rgb255 255 208 70


buttonForegroudColor =
    rgb255 33 37 41


backgroundColor =
    rgb255 44 46 52


foregroundColor =
    rgb255 0 255 0


foregroundColor2 =
    rgb255 216 121 121


font =
    Font.external
        { name = "DotGothic16"
        , url = "https://fonts.googleapis.com/css2?family=DotGothic16"
        }


robotoFont =
    Font.external
        { name = "Roboto"
        , url = "https://fonts.googleapis.com/css2?family=Roboto:wght@700"
        }


lightningSvg : Element msg
lightningSvg =
    Svg.svg
        [ Svg.Attributes.width "24"
        , Svg.Attributes.height "24"
        , Svg.Attributes.viewBox "0 0 24 24"
        ]
        [ Svg.path [ Svg.Attributes.d "M19 10.1907L8.48754 21L12.6726 12.7423H5L14.6157 3L11.5267 10.2835L19 10.1907Z" ] []
        ]
        |> html
