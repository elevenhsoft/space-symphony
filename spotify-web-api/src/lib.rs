use rspotify::{scopes, AuthCodeSpotify, Credentials, OAuth};

#[derive(Default)]
pub struct SpotifyWeb {
    pub spotify: AuthCodeSpotify,
    pub creds: Credentials,
    pub auth_url: String,
}

impl SpotifyWeb {
    pub fn new() -> SpotifyWeb {
        let creds = Credentials::from_env().unwrap();
        let oauth = OAuth {
            redirect_uri: "http://127.0.0.1:8088/success".to_string(),
            scopes: scopes!("user-read-recently-played"),
            ..Default::default()
        };
        let spotify = AuthCodeSpotify::new(creds.clone(), oauth);
        let auth_url = spotify.get_authorize_url(false).unwrap();

        SpotifyWeb {
            spotify,
            creds,
            auth_url,
        }
    }
}
