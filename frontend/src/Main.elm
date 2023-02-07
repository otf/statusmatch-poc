module Main exposing (..)

import Browser
import Element exposing (Element, alignRight, centerY, el, fill, padding, rgb255, row, spacing, text, width)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Html exposing (Html)
import Http


type alias Model =
    { hello : Maybe String
    }


type Msg
    = GotHello (Result Http.Error String)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotHello (Ok hello) ->
            ( { model
                | hello = Just hello
              }
            , Cmd.none
            )

        GotHello (Err _) ->
            ( model, Cmd.none )


init : () -> ( Model, Cmd Msg )
init () =
    ( { hello = Nothing }
    , Http.get
        { url = "api/hello"
        , expect = Http.expectString GotHello
        }
    )


subscriptions : Model -> Sub msg
subscriptions model =
    Sub.none


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


view : Model -> Html msg
view model =
    Element.layout []
        (myRowOfStuff model)


myRowOfStuff : Model -> Element msg
myRowOfStuff model =
    row [ width fill, centerY, spacing 30 ]
        [ myElement model
        , myElement model
        , el [ alignRight ] <| myElement model
        ]


myElement : Model -> Element msg
myElement model =
    el
        [ Background.color (rgb255 240 0 245)
        , Font.color (rgb255 255 255 255)
        , Border.rounded 3
        , padding 30
        ]
        (text (model.hello |> Maybe.withDefault "api error"))
