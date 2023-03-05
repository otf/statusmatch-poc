module Api exposing (..)

import Http
import Json.Decode as D


type alias Challenge =
    String


type alias AccessToken =
    String


type alias TokenType =
    String


type alias Auth =
    { accessToken : AccessToken
    , tokenType : TokenType
    }


type alias Lnurl =
    String


type alias LnurlAuth =
    { lnurl : Lnurl
    , k1 : Challenge
    }


type alias Program_ =
    { id : Int
    , name : String
    }


type alias Status =
    { level : Int
    , name : String
    }


type alias UserStatus =
    { program : Program_
    , status : Status
    }


type alias Link =
    { program : String
    , status : String
    }


lnurlAuthDecoder : D.Decoder LnurlAuth
lnurlAuthDecoder =
    D.map2 LnurlAuth
        (D.field "lnurl" D.string)
        (D.field "k1" D.string)


authorizationDecoder : D.Decoder Auth
authorizationDecoder =
    D.map2 Auth
        (D.field "access_token" D.string)
        (D.field "token_type" D.string)


programDecoder : D.Decoder Program_
programDecoder =
    D.map2 Program_
        (D.field "id" D.int)
        (D.field "name" D.string)


programListDecoder : D.Decoder (List Program_)
programListDecoder =
    D.list programDecoder


statusDecoder : D.Decoder Status
statusDecoder =
    D.map2 Status
        (D.field "level" D.int)
        (D.field "name" D.string)


statusListDecoder : D.Decoder (List Status)
statusListDecoder =
    D.list statusDecoder


linkDecoder : D.Decoder Link
linkDecoder =
    D.map2 Link
        (D.field "program" D.string)
        (D.field "status" D.string)


linkListDecoder : D.Decoder (List Link)
linkListDecoder =
    D.list linkDecoder


userStatusDecoder : D.Decoder UserStatus
userStatusDecoder =
    D.map2 UserStatus
        (D.field "program" programDecoder)
        (D.field "status" statusDecoder)


userStatusListDecoder : D.Decoder (List UserStatus)
userStatusListDecoder =
    D.list userStatusDecoder


fetchLnurlAuth : (Result Http.Error LnurlAuth -> msg) -> Cmd msg
fetchLnurlAuth tagger =
    Http.get
        { url = "api/login"
        , expect = Http.expectJson tagger lnurlAuthDecoder
        }


fetchLnurlAuthState : LnurlAuth -> (Result Http.Error Auth -> msg) -> Cmd msg
fetchLnurlAuthState { k1 } tagger =
    Http.get
        { url = "api/login/" ++ k1
        , expect = Http.expectJson tagger authorizationDecoder
        }


fetchPrograms : String -> (Result Http.Error (List Program_) -> msg) -> Cmd msg
fetchPrograms text tagger =
    Http.get
        { url = "api/programs/search?text=" ++ text
        , expect = Http.expectJson tagger programListDecoder
        }


fetchStatuses : Program_ -> (Result Http.Error (List Status) -> msg) -> Cmd msg
fetchStatuses program tagger =
    let
        id =
            String.fromInt program.id

        url =
            "api/programs/" ++ id ++ "/statuses"
    in
    Http.get
        { url = url
        , expect = Http.expectJson tagger statusListDecoder
        }


fetchLinks : Program_ -> Status -> (Result Http.Error (List Link) -> msg) -> Cmd msg
fetchLinks program status tagger =
    let
        id =
            String.fromInt program.id

        level =
            String.fromInt status.level

        url =
            "api/programs/" ++ id ++ "/statuses/" ++ level ++ "/links"
    in
    Http.get
        { url = url
        , expect = Http.expectJson tagger linkListDecoder
        }


fetchUserStatuses : Auth -> (Result Http.Error (List UserStatus) -> msg) -> Cmd msg
fetchUserStatuses { tokenType, accessToken } tagger =
    Http.request
        { method = "GET"
        , headers =
            [ Http.header "Authorization" (tokenType ++ " " ++ accessToken)
            ]
        , url = "api/user/statuses"
        , expect = Http.expectJson tagger userStatusListDecoder
        , body = Http.emptyBody
        , timeout = Nothing
        , tracker = Nothing
        }
