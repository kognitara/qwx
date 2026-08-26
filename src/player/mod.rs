//! # QWX Music & Spotify Player Module
//!
//! Terminal Music Player with full Spotify Web API integration and extensible local disk audio playback architecture.
//!
//! Features:
//! - Spotify Web API client (Search, Playback control, Devices, Playlists, Tracks, Albums, Queue).
//! - Authentication via OAuth User Access Token or Client Credentials.
//! - Playback controls: Play, Pause, Next, Previous, Seek, Volume, Shuffle, Repeat.
//! - Spotify Connect device discovery and playback transfer.
//! - Terminal UI with multiple tabs: Now Playing, Search, Queue, Devices, Playlists, Settings.
//! - Live audio visualizer / ASCII waveform rendering.
//! - Extensible `AudioSource` design ready for local file playback (MP3/FLAC/WAV/OGG).

use crossterm::cursor::{Hide, MoveTo};
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::{execute, queue};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Audio Source for playback: Spotify streaming URI or local file on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSource {
    Spotify { uri: String, id: String },
    LocalFile { path: PathBuf },
}

/// Represents a track item containing detailed metadata about a music track.
///
/// # Fields
///
/// - `id`
///   The unique identifier for the track. Typically provided by the source service.
///
/// - `name`
///   The name or title of the track.
///
/// - `artists`
///   A list of artist names associated with the track.
///
/// - `album_name`
///   The name of the album the track belongs to.
///
/// - `duration_ms`
///   The duration of the track in milliseconds.
///
/// - `uri`
///   The Uniform Resource Identifier (URI) pointing to the track. This is used to uniquely identify and locate the track within the source service.
///
/// - `preview_url`
///   An optional field containing a URL to a preview snippet of the track, if available.
///
/// - `popularity`
///   An optional field representing the track's popularity as a numeric score. The range and meaning of this value depend on the source service.
///
/// - `is_playable`
///   A boolean indicating whether the track is playable by the service.
///
/// - `source`
///   The source of the audio, represented by the `AudioSource` enum or struct. This typically identifies the origin of the track.
///
/// # Traits
///
/// - `Debug`: Allows instances of `TrackItem` to be formatted using the `{:?}` formatter.
/// - `Clone`: Enables the `TrackItem` to be cloned, creating an independent copy.
/// - `PartialEq`: Supports equality comparisons between two `TrackItem` instances.
/// - `Eq`: Specifies that `TrackItem` implements reflexive equality.
/// - `Serialize`: Allows the `TrackItem` to be serialized (used for converting the struct to formats like JSON).
/// - `Deserialize`: Allows the `TrackItem` to be deserialized (used for reconstructing the struct from serialized formats).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackItem {
    pub id: String,
    pub name: String,
    pub artists: Vec<String>,
    pub album_name: String,
    pub duration_ms: u64,
    pub uri: String,
    pub preview_url: Option<String>,
    pub popularity: Option<u32>,
    pub is_playable: bool,
    pub source: AudioSource,
}

impl TrackItem {
    /// Returns a string representation of the artists associated with the object.
    ///
    /// If the `artists` collection is empty, the method returns `"Unknown Artist"`.
    /// Otherwise, it joins the artist names with a comma separator and returns the resulting string.
    ///
    /// # Returns
    /// - A `String` containing either "Unknown Artist" if no artists are present, or
    ///   a comma-separated list of artist names.
    pub fn artists_str(&self) -> String {
        if self.artists.is_empty() {
            "Unknown Artist".to_string()
        } else {
            self.artists.join(", ")
        }
    }
    /// Formats the duration of the track in a human-readable format.
    pub fn formatted_duration(&self) -> String {
        format_duration_ms(self.duration_ms)
    }
}

/// A struct representing an item on an album, containing metadata and details about the album.
///
/// # Fields
///
/// * `id` - A unique identifier for the album item.
/// * `name` - The name of the album.
/// * `artists` - A list of artist names associated with the album.
/// * `release_date` - The release date of the album, represented as a string.
/// * `total_tracks` - The total number of tracks on the album.
/// * `uri` - A uniform resource identifier (URI) for the album, typically used for linking.
///
/// # Traits
///
/// This struct derives the following traits:
/// * `Debug` - Enables formatting the struct using `{:?}` for debugging purposes.
/// * `Clone` - Allows the struct to be cloned, creating a duplicate with the same data.
/// * `PartialEq` - Enables comparison for equality between two `AlbumItem` instances.
/// * `Eq` - Ensures strict equality comparison for `AlbumItem`.
/// * `Serialize` - Allows the struct to be serialized into formats like JSON or binary.
/// * `Deserialize` - Allows the struct to be deserialized from formats like JSON or binary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlbumItem {
    pub id: String,
    pub name: String,
    pub artists: Vec<String>,
    pub release_date: String,
    pub total_tracks: u32,
    pub uri: String,
}

impl AlbumItem {
    pub fn artists_str(&self) -> String {
        if self.artists.is_empty() {
            "Various Artists".to_string()
        } else {
            self.artists.join(", ")
        }
    }
}

/// Represents an item in a playlist.
///
/// This struct is used to model the data for a single item within a playlist, including its
/// identifying attributes and metadata. It is designed to be serializable and deserializable,
/// allowing for easy storage and transfer of playlist item data.
///
/// # Fields
/// - `id` (`String`): A unique identifier for the playlist item.
/// - `name` (`String`): The name of the playlist item.
/// - `description` (`String`): A brief description of the playlist item.
/// - `owner_name` (`String`): The name of the owner or creator of the playlist item.
/// - `total_tracks` (`u32`): The total number of tracks in the playlist.
/// - `uri` (`String`): A URI (Uniform Resource Identifier) that uniquely identifies the playlist item.
///
/// # Traits Implemented
/// - `Debug`: Enables formatted output and debugging capabilities.
/// - `Clone`: Allows for creating deep copies of `PlaylistItem`.
/// - `PartialEq`: Enables comparison of two `PlaylistItem` objects for equality.
/// - `Eq`: Ensures strict equality comparison.
/// - `Serialize`: Allows the struct to be converted into a format suitable for storage or communication.
/// - `Deserialize`: Allows the struct to be reconstructed from serialized data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_name: String,
    pub total_tracks: u32,
    pub uri: String,
}

///
/// `DeviceItem` is a data structure that represents a device with its associated metadata.
/// It implements the `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize` traits,
/// allowing for easy debugging, cloning, data comparison, and serialization/deserialization.
///
/// # Fields
///
/// * `id` (`String`): A unique identifier for the device.
///
/// * `is_active` (`bool`): Indicates whether the device is currently active.
///
/// * `is_restricted` (`bool`): Indicates whether the device is restricted or unavailable for certain actions.
///
/// * `name` (`String`): The human-readable name of the device.
///
/// * `device_type` (`String`): The type or category of the device (e.g., "Smartphone", "Speaker").
///
/// * `volume_percent` (`Option<u32>`): The current volume level of the device as a percentage (0-100).
///   If `None`, the volume level is unknown or not applicable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceItem {
    pub id: String,
    pub is_active: bool,
    pub is_restricted: bool,
    pub name: String,
    pub device_type: String,
    pub volume_percent: Option<u32>,
}

/// Represents the playback state of a media player, including information about the currently playing track,
/// player settings, and playback progress.
///
/// # Fields
///
/// * `is_playing` - A boolean indicating whether the media is currently playing (`true`) or paused (`false`).
/// * `progress_ms` - The current playback position in milliseconds.
/// * `item` - An `Option` containing the currently playing track as a `TrackItem`. If no track is playing, this will be `None`.
/// * `device` - An `Option` containing the device information as a `DeviceItem` on which playback is occurring. If no device is active, this will be `None`.
/// * `shuffle_state` - A boolean indicating whether shuffle mode is enabled (`true`) or disabled (`false`).
/// * `repeat_state` - A `RepeatMode` enum value representing the current repeat setting (e.g., off, track, or context).
/// * `volume_percent` - The volume level of the playback device as a percentage (0 to 100).
/// * `last_synced_at` - An `Option` containing the `Instant` when the playback state was last synced. This field is not serialized/deserialized as it is marked with `#[serde(skip)]`.
///
/// # Derives
///
/// The struct derives the following traits:
///
/// * `Debug` - Allows the struct to be formatted using the `{:?}` formatter for debugging purposes.
/// * `Clone` - Enables cloning of the struct for creating copies.
/// * `Serialize` - Enables serialization of the struct into formats such as JSON.
/// * `Deserialize` - Enables deserialization of the struct from serialized formats.
///
/// This struct is typically used to encapsulate the state of a media player's playback session, useful for tracking
/// and controlling playback in applications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub progress_ms: u64,
    pub item: Option<TrackItem>,
    pub device: Option<DeviceItem>,
    pub shuffle_state: bool,
    pub repeat_state: RepeatMode,
    pub volume_percent: u32,
    #[serde(skip)]
    pub last_synced_at: Option<Instant>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            progress_ms: 0,
            item: None,
            device: None,
            shuffle_state: false,
            repeat_state: RepeatMode::Off,
            volume_percent: 75,
            last_synced_at: None,
        }
    }
}

/// An enumeration representing the repeat modes available for media playback.
///
/// The `RepeatMode` enum defines three possible states that specify how media
/// playback should behave with regard to repeating content. This is often used
/// in media players or streaming applications.
///
/// # Variants
///
/// * `Off` - Repeat is disabled, i.e., content will play through once and stop.
/// * `Track` - The current track is repeated indefinitely.
/// * `Context` - The entire context (e.g., playlist or album) is repeated indefinitely.
///
/// # Derives
///
/// This enum derives several traits:
///
/// * `Debug` - Allows formatting the value using the debug formatter.
/// * `Clone` - Enables creating a copy of the value.
/// * `Copy` - Allows for bitwise copying of the value.
/// * `PartialEq` - Enables equality and inequality comparisons between values.
/// * `Eq` - Ensures that equality is strict, with no partial equivalence.
/// * `Serialize` - Allows the enum to be serialized (commonly used with formats like JSON).
/// * `Deserialize` - Enables the enum to be deserialized from a serialized format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    Track,
    Context,
}

impl RepeatMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RepeatMode::Off => "off",
            RepeatMode::Track => "track",
            RepeatMode::Context => "context",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::Context,
            RepeatMode::Context => RepeatMode::Track,
            RepeatMode::Track => RepeatMode::Off,
        }
    }
}

/// Represents the different tabs or sections available in a media player application.
///
/// The `PlayerTab` enum is annotated with several useful traits, such as:
/// - `Debug`: Allows instances of `PlayerTab` to be formatted using the `{:?}` formatter.
/// - `Clone`: Enables the creation of a duplicate copy of a `PlayerTab` instance.
/// - `Copy`: Allows `PlayerTab` instances to be implicitly copied instead of moved.
/// - `PartialEq` and `Eq`: Facilitates comparison of `PlayerTab` instances for equality.
/// - `Serialize` and `Deserialize`: Provides support for serializing and deserializing `PlayerTab`
///   instances, typically for use with formats like JSON.
///
/// ## Variants
/// - `NowPlaying`: Represents the "Now Playing" tab, typically showing the currently playing media.
/// - `Search`: Represents the search tab, allowing users to search for media content.
/// - `Queue`: Represents the queue tab, displaying the list of media items in the playback queue.
/// - `Playlists`: Represents the playlist tab, providing access to the user's playlists.
/// - `Devices`: Represents the device tab, allowing users to view and manage playback devices.
/// - `Config`: Represents the configuration tab, where users can modify application settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerTab {
    NowPlaying,
    Search,
    Queue,
    Playlists,
    Devices,
    Config,
}

impl PlayerTab {
    pub fn all() -> &'static [PlayerTab] {
        &[
            PlayerTab::NowPlaying,
            PlayerTab::Search,
            PlayerTab::Queue,
            PlayerTab::Playlists,
            PlayerTab::Devices,
            PlayerTab::Config,
        ]
    }
    /// Returns the title corresponding to each variant of the `PlayerTab` enum.
    ///
    /// # Description
    /// This method provides a human-readable title for each tab in the player interface.
    /// The returned title is a static string slice that matches the purpose of the specific tab.
    ///
    /// # Returns
    /// A `&static str` representing the title of the tab.
    ///
    /// # Variants and Corresponding Titles
    /// - `PlayerTab::NowPlaying` => `"Now Playing"`
    /// - `PlayerTab::Search` => `"Search & Explore"`
    /// - `PlayerTab::Queue` => `"Play Queue"`
    /// - `PlayerTab::Playlists` => `"Playlists & Albums"`
    /// - `PlayerTab::Devices` => `"Connect Devices"`
    /// - `PlayerTab::Config` => `"Spotify Auth & Settings"`
    pub fn title(&self) -> &'static str {
        match self {
            PlayerTab::NowPlaying => "Now Playing",
            PlayerTab::Search => "Search & Explore",
            PlayerTab::Queue => "Play Queue",
            PlayerTab::Playlists => "Playlists & Albums",
            PlayerTab::Devices => "Connect Devices",
            PlayerTab::Config => "Spotify Auth & Settings",
        }
    }

    pub fn shortcut(&self) -> char {
        match self {
            PlayerTab::NowPlaying => '1',
            PlayerTab::Search => '2',
            PlayerTab::Queue => '3',
            PlayerTab::Playlists => '4',
            PlayerTab::Devices => '5',
            PlayerTab::Config => '6',
        }
    }
}

/// Represents different categories that can be used for searching in a music-related context.
///
/// # Variants
///
/// - `Tracks`: Represents a search category for individual tracks or songs.
/// - `Albums`: Represents a search category for complete albums.
/// - `Playlists`: Represents a search category for curated playlists.
/// - `Artists`: Represents a search category for music artists.
///
/// # Derives
/// - `Debug`: Allows instances of `SearchCategory` to be formatted using the `{:?}` formatter.
/// - `Clone`: Provides the ability to produce a copy of a `SearchCategory` instance.
/// - `Copy`: Allows for bitwise copying of `SearchCategory` values.
/// - `PartialEq`: Enables comparison of `SearchCategory` instances for equality.
/// - `Eq`: Ensures reflexive, symmetric, and transitive equality comparisons.
/// - `Serialize`: Allows `SearchCategory` to be serialized, e.g., for JSON or other forms of storage or communication.
/// - `Deserialize`: Allows for deserialization of `SearchCategory` from serialized formats, such as JSON.
///
/// This enum is used to narrow down or filter searches based on a specific type of entity in a music domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchCategory {
    Tracks,
    Albums,
    Playlists,
    Artists,
}

impl SearchCategory {
    pub fn all() -> &'static [SearchCategory] {
        &[
            SearchCategory::Tracks,
            SearchCategory::Albums,
            SearchCategory::Playlists,
            SearchCategory::Artists,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SearchCategory::Tracks => "Tracks",
            SearchCategory::Albums => "Albums",
            SearchCategory::Playlists => "Playlists",
            SearchCategory::Artists => "Artists",
        }
    }

    pub fn api_type(&self) -> &'static str {
        match self {
            SearchCategory::Tracks => "track",
            SearchCategory::Albums => "album",
            SearchCategory::Playlists => "playlist",
            SearchCategory::Artists => "artist",
        }
    }
}

/// A data structure representing the results of a search query.
///
/// The `SearchResults` struct contains collections of tracks, albums, and playlists
/// that match the search criteria. It is designed to provide an organized and convenient
/// way to access these results.
///
/// # Fields
///
/// * `tracks` - A vector containing `TrackItem` objects that represent the tracks
///   matching the search query.
/// * `albums` - A vector containing `AlbumItem` objects that represent the albums
///   matching the search query.
/// * `playlists` - A vector containing `PlaylistItem` objects that represent the playlists
///   matching the search query.
///
/// # Derives
///
/// * `Debug` - Enables formatting the struct using the `{:?}` formatter, which
///   is helpful for debugging purposes.
/// * `Clone` - Allows creating an exact copy of a `SearchResults` instance.
/// * `Default` - Provides a default implementation for creating an empty `SearchResults` instance
///   where each field is initialized to an empty vector.
#[derive(Debug, Clone, Default)]
pub struct SearchResults {
    pub tracks: Vec<TrackItem>,
    pub albums: Vec<AlbumItem>,
    pub playlists: Vec<PlaylistItem>,
}

impl SearchResults {
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty() && self.albums.is_empty() && self.playlists.is_empty()
    }

    pub fn total_count(&self, category: SearchCategory) -> usize {
        match category {
            SearchCategory::Tracks => self.tracks.len(),
            SearchCategory::Albums => self.albums.len(),
            SearchCategory::Playlists => self.playlists.len(),
            SearchCategory::Artists => 0,
        }
    }
}

/// Represents the credentials required to authenticate with the Spotify API.
///
/// This struct is used to store and manage the necessary tokens and client information
/// for making authorized requests to Spotify's APIs. It implements the `Debug`, `Clone`,
/// `Serialize`, and `Deserialize` traits to support debugging, copying, and (de)serializing
/// operations.
///
/// # Fields
///
/// * `access_token` - Optional access token used to authenticate API requests.
/// * `refresh_token` - Optional token used to refresh the access token when it expires.
/// * `client_id` - Optional client identifier associated with the Spotify application.
/// * `client_secret` - Optional client secret for the Spotify application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyCredentials {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

impl Default for SpotifyCredentials {
    fn default() -> Self {
        Self {
            access_token: std::env::var("SPOTIFY_TOKEN")
                .or_else(|_| std::env::var("SPOTIFY_ACCESS_TOKEN"))
                .ok(),
            refresh_token: std::env::var("SPOTIFY_REFRESH_TOKEN").ok(),
            client_id: std::env::var("SPOTIFY_CLIENT_ID").ok(),
            client_secret: std::env::var("SPOTIFY_CLIENT_SECRET").ok(),
        }
    }
}

impl SpotifyCredentials {
    pub fn load_from_config() -> Self {
        let mut creds = Self::default();
        if let Some(config_dir) = dirs_config_path() {
            let config_file = config_dir.join("spotify.json");
            if config_file.exists() {
                if let Ok(content) = fs::read_to_string(&config_file) {
                    if let Ok(loaded) = serde_json::from_str::<SpotifyCredentials>(&content) {
                        if creds.access_token.is_none() {
                            creds.access_token = loaded.access_token;
                        }
                        if creds.refresh_token.is_none() {
                            creds.refresh_token = loaded.refresh_token;
                        }
                        if creds.client_id.is_none() {
                            creds.client_id = loaded.client_id;
                        }
                        if creds.client_secret.is_none() {
                            creds.client_secret = loaded.client_secret;
                        }
                    }
                }
            }
        }
        creds
    }
    /// Saves the current object state to a configuration file in the user's configuration directory.
    ///
    /// This function serializes the object implementing this method into a JSON file
    /// and writes it to a standard configuration directory (e.g., `$XDG_CONFIG_HOME` on Linux,
    /// `%APPDATA%` on Windows, etc.). The file is named `spotify.json` and is stored
    /// in the directory returned by the `dirs_config_path` function. If the directory does
    /// not exist, it will be created.
    ///
    /// # Errors
    /// Returns an `io::Result<()>` which may contain an error in the following cases:
    /// - If the configuration directory path is invalid or inaccessible.
    /// - If the creation of the directory fails.
    /// - If serialization of the object into JSON format fails.
    /// - If writing the serialized JSON to the file fails.
    ///
    /// # Dependencies
    /// - The function relies on the `dirs_config_path` function to determine the configuration directory.
    /// - JSON serialization is performed using the `serde_json` crate.
    /// - Uses the `std::fs` module for file operations.
    pub fn save_to_config(&self) -> io::Result<()> {
        if let Some(config_dir) = dirs_config_path() {
            fs::create_dir_all(&config_dir)?;
            let config_file = config_dir.join("spotify.json");
            let json = serde_json::to_string_pretty(self)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            fs::write(config_file, json)?;
        }
        Ok(())
    }
    /// Checks if the configuration for an instance is complete.
    ///
    /// This method evaluates whether the necessary credentials are available
    /// for the functionality to operate correctly. The configuration is considered
    /// complete if one of the following conditions is met:
    ///
    /// 1. An access token is set (`self.access_token` is `Some`).
    /// 2. Both a client ID and client secret are set (`self.client_id` and `self.client_secret` are `Some`).
    ///
    /// # Returns
    ///
    /// * `true` - If the configuration is complete.
    /// * `false` - If the configuration is incomplete.
    pub fn is_configured(&self) -> bool {
        self.access_token.is_some() || (self.client_id.is_some() && self.client_secret.is_some())
    }

    /// Generates the file path for the Spotify configuration file.
    ///
    /// This function retrieves the user's configuration directory path using the `dirs_config_path`
    /// function and appends the filename "spotify.json" to it. If the configuration directory path
    /// exists, it returns the full path to the configuration file as a `PathBuf`.
    ///
    /// # Returns
    /// - `Some(PathBuf)` containing the full path to "spotify.json" if the configuration directory exists.
    /// - `None` if the configuration directory cannot be determined.
    ///
    pub fn config_file_path() -> Option<PathBuf> {
        dirs_config_path().map(|dir| dir.join("spotify.json"))
    }

    /// Updates tokens (access_token and/or refresh_token) in the saved configuration file.
    ///
    /// Loads existing configuration first, overrides with provided token values (if Some),
    /// writes to disk, and returns the updated `SpotifyCredentials`.
    pub fn update_tokens_in_config(
        access_token: Option<String>,
        refresh_token: Option<String>,
    ) -> io::Result<Self> {
        let mut creds = Self::load_from_config();
        if access_token.is_some() {
            creds.access_token = access_token;
        }
        if refresh_token.is_some() {
            creds.refresh_token = refresh_token;
        }
        creds.save_to_config()?;
        Ok(creds)
    }

    /// Updates the access token and persists it to the configuration file.
    pub fn update_access_token_in_config(access_token: impl Into<String>) -> io::Result<Self> {
        Self::update_tokens_in_config(Some(access_token.into()), None)
    }

    /// Updates the refresh token and persists it to the configuration file.
    pub fn update_refresh_token_in_config(refresh_token: impl Into<String>) -> io::Result<Self> {
        Self::update_tokens_in_config(None, Some(refresh_token.into()))
    }
}

/// Returns the configuration path for the application.
///
/// The function first attempts to retrieve the user's configuration directory
/// using the `dirs::config_dir` function, which provides a platform-appropriate
/// default configuration directory (e.g., `~/.config` on Linux or
/// `%AppData%` on Windows). If this directory is found, a folder named "qwx" is
/// appended to it and returned as a `PathBuf`.
///
/// If `dirs::config_dir` does not return a valid path, the function falls back
/// to using the `HOME` environment variable. The `HOME` environment variable
/// is used to construct a fallback configuration path in the form:
/// `<HOME>/.config/qwx`.
///
/// # Returns
/// * `Some(PathBuf)` - The computed configuration path if the process succeeds.
/// * `None` - If no configuration path could be determined.
fn dirs_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("qwx")).or_else(|| {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".config").join("qwx"))
    })
}

/// Represents a client for interacting with the Spotify Web API.
///
/// The `SpotifyClient` struct provides functionality to communicate with Spotify's
/// API using `reqwest::blocking::Client`. Credentials and base URL for the API are
/// required to perform operations.
///
/// # Fields
/// * `client` - An instance of `reqwest::blocking::Client` used to make HTTP requests.
/// * `credentials` - A `SpotifyCredentials` struct that holds the Spotify API credentials
///   (such as client ID and client secret).
/// * `base_url` - A `String` representing the base URL of the Spotify API (e.g., "https://api.spotify.com").
///
/// # Examples
///
/// Create a new instance of the `SpotifyClient`:
/// ```rust
/// let spotify_client = SpotifyClient {
///     client: reqwest::blocking::Client::new(),
///     credentials: SpotifyCredentials {
///         client_id: "your_client_id".to_string(),
///         client_secret: "your_client_secret".to_string(),
///     },
///     base_url: "https://api.spotify.com".to_string(),
/// };
/// ```
///
/// Use the `SpotifyClient` to make API calls:
#[derive(Debug, Clone)]
pub struct SpotifyClient {
    client: reqwest::blocking::Client,
    pub credentials: SpotifyCredentials,
    pub base_url: String,
}

impl Default for SpotifyClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotifyClient {
    #[must_use]
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        let credentials = SpotifyCredentials::load_from_config();

        Self {
            client,
            credentials,
            base_url: "https://api.spotify.com/v1".to_string(),
        }
    }

    /// Configures the client with the specified access token.
    ///
    /// This method allows you to set an access token for the client, which will be
    /// used for authentication purposes in later API requests. It consumes
    /// the provided value and converts it into a `String` before assigning it to
    /// the client's credentials.
    ///
    /// # Arguments
    ///
    /// * `token` - A value that can be converted into a `String`, representing the
    ///   access token to be used by the client.
    ///
    /// # Returns
    ///
    /// Returns an instance of the client with the access token configured.
    pub fn with_token(token: impl Into<String>) -> Self {
        let mut client = Self::new();
        client.credentials.access_token = Some(token.into());
        client
    }

    /// Sets the access token.
    pub fn set_token(&mut self, token: impl Into<String>) {
        self.credentials.access_token = Some(token.into());
    }

    /// Sets the access token and immediately saves it to the configuration file.
    pub fn set_and_save_token(&mut self, token: impl Into<String>) -> io::Result<()> {
        let tok = token.into();
        self.credentials.access_token = Some(tok);
        self.credentials.save_to_config()
    }

    /// Fetches Client Credentials Access Token if Client ID and Secret are configured.
    pub fn request_client_credentials_token(&mut self) -> Result<String, String> {
        let client_id = self
            .credentials
            .client_id
            .as_ref()
            .ok_or_else(|| "Missing Client ID".to_string())?;
        let client_secret = self
            .credentials
            .client_secret
            .as_ref()
            .ok_or_else(|| "Missing Client Secret".to_string())?;

        let res = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(client_id, Some(client_secret))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials")
            .send()
            .map_err(|e| format!("HTTP request error: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Auth failed with status: {}", res.status()));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let token_data: TokenResponse = res
            .json()
            .map_err(|e| format!("JSON parsing error: {}", e))?;

        self.credentials.access_token = Some(token_data.access_token.clone());
        let _ = self.credentials.save_to_config();
        Ok(token_data.access_token)
    }

    /// Prepares authorized request builder.
    fn auth_get(&self, path: &str) -> Result<reqwest::blocking::RequestBuilder, String> {
        let token =
            self.credentials.access_token.as_ref().ok_or_else(|| {
                "Spotify Access Token not set. Press [t] to configure.".to_string()
            })?;
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.get(&url).bearer_auth(token))
    }

    fn auth_post(&self, path: &str) -> Result<reqwest::blocking::RequestBuilder, String> {
        let token =
            self.credentials.access_token.as_ref().ok_or_else(|| {
                "Spotify Access Token not set. Press [t] to configure.".to_string()
            })?;
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.post(&url).bearer_auth(token))
    }

    fn auth_put(&self, path: &str) -> Result<reqwest::blocking::RequestBuilder, String> {
        let token =
            self.credentials.access_token.as_ref().ok_or_else(|| {
                "Spotify Access Token not set. Press [t] to configure.".to_string()
            })?;
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.put(&url).bearer_auth(token))
    }

    /// Search tracks, albums, and playlists on Spotify.
    pub fn search(&self, query: &str, limit: u32) -> Result<SearchResults, String> {
        if query.trim().is_empty() {
            return Ok(SearchResults::default());
        }

        // Spotify API /v1/search enforces a limit between 1 and 10 (maximum 10 items per type)
        let limit = limit.clamp(1, 10);
        let encoded_q = urlencoding_simple(query);
        let path = format!(
            "/search?q={}&type=track,album,playlist&limit={}",
            encoded_q, limit
        );

        let req = self.auth_get(&path)?;
        let res = req
            .send()
            .map_err(|e| format!("Search network error: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().unwrap_or_default();
            return Err(format!("Spotify API Error ({}): {}", status, err_body));
        }

        let val: serde_json::Value = res
            .json()
            .map_err(|e| format!("Search response JSON error: {}", e))?;

        let mut results = SearchResults::default();

        // Parse tracks
        if let Some(items) = val["tracks"]["items"].as_array() {
            for item in items {
                if let Some(track) = parse_spotify_track_json(item) {
                    results.tracks.push(track);
                }
            }
        }

        // Parse albums
        if let Some(items) = val["albums"]["items"].as_array() {
            for item in items {
                let id = item["id"].as_str().unwrap_or_default().to_string();
                let name = item["name"].as_str().unwrap_or_default().to_string();
                let uri = item["uri"].as_str().unwrap_or_default().to_string();
                let release_date = item["release_date"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                let total_tracks = item["total_tracks"].as_u64().unwrap_or(0) as u32;
                let mut artists = Vec::new();
                if let Some(artists_arr) = item["artists"].as_array() {
                    for a in artists_arr {
                        if let Some(a_name) = a["name"].as_str() {
                            artists.push(a_name.to_string());
                        }
                    }
                }
                results.albums.push(AlbumItem {
                    id,
                    name,
                    artists,
                    release_date,
                    total_tracks,
                    uri,
                });
            }
        }

        // Parse playlists
        if let Some(items) = val["playlists"]["items"].as_array() {
            for item in items {
                if item.is_null() {
                    continue;
                }
                let id = item["id"].as_str().unwrap_or_default().to_string();
                let name = item["name"].as_str().unwrap_or_default().to_string();
                let uri = item["uri"].as_str().unwrap_or_default().to_string();
                let description = item["description"].as_str().unwrap_or_default().to_string();
                let owner_name = item["owner"]["display_name"]
                    .as_str()
                    .unwrap_or("Spotify")
                    .to_string();
                let total_tracks = item["tracks"]["total"].as_u64().unwrap_or(0) as u32;

                results.playlists.push(PlaylistItem {
                    id,
                    name,
                    description,
                    owner_name,
                    total_tracks,
                    uri,
                });
            }
        }

        Ok(results)
    }

    /// Gets current playback state from Spotify Connect.
    pub fn get_playback_state(&self) -> Result<Option<PlaybackState>, String> {
        let req = self.auth_get("/me/player")?;
        let res = req
            .send()
            .map_err(|e| format!("Playback state error: {}", e))?;

        if res.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        if !res.status().is_success() {
            return Err(format!("Get playback failed: {}", res.status()));
        }

        let val: serde_json::Value = res
            .json()
            .map_err(|e| format!("Playback JSON error: {}", e))?;

        let is_playing = val["is_playing"].as_bool().unwrap_or(false);
        let progress_ms = val["progress_ms"].as_u64().unwrap_or(0);
        let shuffle_state = val["shuffle_state"].as_bool().unwrap_or(false);
        let repeat_str = val["repeat_state"].as_str().unwrap_or("off");
        let repeat_state = match repeat_str {
            "track" => RepeatMode::Track,
            "context" => RepeatMode::Context,
            _ => RepeatMode::Off,
        };

        let mut track_item = None;
        if let Some(item_obj) = val.get("item").filter(|i| !i.is_null()) {
            track_item = parse_spotify_track_json(item_obj);
        }

        let mut device_item = None;
        let mut volume_percent = 75;
        if let Some(dev_obj) = val.get("device").filter(|d| !d.is_null()) {
            let id = dev_obj["id"].as_str().unwrap_or_default().to_string();
            let name = dev_obj["name"].as_str().unwrap_or_default().to_string();
            let is_active = dev_obj["is_active"].as_bool().unwrap_or(false);
            let is_restricted = dev_obj["is_restricted"].as_bool().unwrap_or(false);
            let device_type = dev_obj["type"].as_str().unwrap_or("Speaker").to_string();
            let vol = dev_obj["volume_percent"].as_u64().map(|v| v as u32);
            if let Some(v) = vol {
                volume_percent = v;
            }
            device_item = Some(DeviceItem {
                id,
                is_active,
                is_restricted,
                name,
                device_type,
                volume_percent: vol,
            });
        }

        Ok(Some(PlaybackState {
            is_playing,
            progress_ms,
            item: track_item,
            device: device_item,
            shuffle_state,
            repeat_state,
            volume_percent,
            last_synced_at: Some(Instant::now()),
        }))
    }

    /// Plays a Spotify URI (Track, Album, or Playlist).
    pub fn play_uri(&self, uri: &str, device_id: Option<&str>) -> Result<(), String> {
        let mut path = "/me/player/play".to_string();
        if let Some(dev) = device_id {
            path.push_str(&format!("?device_id={}", dev));
        }

        let mut body = serde_json::Map::new();
        if uri.starts_with("spotify:track:") {
            body.insert(
                "uris".to_string(),
                serde_json::Value::Array(vec![serde_json::Value::String(uri.to_string())]),
            );
        } else {
            body.insert(
                "context_uri".to_string(),
                serde_json::Value::String(uri.to_string()),
            );
        }

        let req = self.auth_put(&path)?.json(&serde_json::Value::Object(body));
        let res = req.send().map_err(|e| format!("Play error: {}", e))?;

        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let err_text = res.text().unwrap_or_default();
            Err(format!("Play request failed: {}", err_text))
        }
    }

    /// Resumes playback on the active device.
    pub fn resume(&self) -> Result<(), String> {
        let req = self.auth_put("/me/player/play")?;
        let res = req.send().map_err(|e| format!("Resume error: {}", e))?;
        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err("Resume failed. Ensure a Spotify Connect device is active.".to_string())
        }
    }

    /// Pauses playback.
    pub fn pause(&self) -> Result<(), String> {
        let req = self.auth_put("/me/player/pause")?;
        let res = req.send().map_err(|e| format!("Pause error: {}", e))?;
        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err("Pause failed".to_string())
        }
    }

    /// Skips to the next track.
    pub fn next(&self) -> Result<(), String> {
        let req = self.auth_post("/me/player/next")?;
        let res = req.send().map_err(|e| format!("Next error: {}", e))?;
        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err("Next track failed".to_string())
        }
    }

    /// Skips to previous track.
    pub fn previous(&self) -> Result<(), String> {
        let req = self.auth_post("/me/player/previous")?;
        let res = req.send().map_err(|e| format!("Previous error: {}", e))?;
        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err("Previous track failed".to_string())
        }
    }

    /// Seeks to position in ms.
    pub fn seek(&self, position_ms: u64) -> Result<(), String> {
        let path = format!("/me/player/seek?position_ms={}", position_ms);
        let req = self.auth_put(&path)?;
        let res = req.send().map_err(|e| format!("Seek error: {}", e))?;
        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err("Seek failed".to_string())
        }
    }

    /// Sets playback volume (0..100).
    pub fn set_volume(&self, volume_percent: u32) -> Result<(), String> {
        let vol = volume_percent.min(100);
        let path = format!("/me/player/volume?volume_percent={}", vol);
        let req = self.auth_put(&path)?;
        let res = req.send().map_err(|e| format!("Volume error: {}", e))?;
        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err("Set volume failed".to_string())
        }
    }

    /// Toggles shuffle mode.
    pub fn set_shuffle(&self, state: bool) -> Result<(), String> {
        let path = format!("/me/player/shuffle?state={}", state);
        let req = self.auth_put(&path)?;
        let res = req.send().map_err(|e| format!("Shuffle error: {}", e))?;
        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err("Toggle shuffle failed".to_string())
        }
    }

    /// Sets repeat mode (off, track, context).
    pub fn set_repeat(&self, mode: RepeatMode) -> Result<(), String> {
        let path = format!("/me/player/repeat?state={}", mode.as_str());
        let req = self.auth_put(&path)?;
        let res = req.send().map_err(|e| format!("Repeat error: {}", e))?;
        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err("Set repeat failed".to_string())
        }
    }

    /// Gets available Spotify Connect devices.
    pub fn get_devices(&self) -> Result<Vec<DeviceItem>, String> {
        let req = self.auth_get("/me/player/devices")?;
        let res = req.send().map_err(|e| format!("Devices error: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Get devices failed: {}", res.status()));
        }

        let val: serde_json::Value = res
            .json()
            .map_err(|e| format!("Devices JSON error: {}", e))?;

        let mut devices = Vec::new();
        if let Some(arr) = val["devices"].as_array() {
            for dev in arr {
                let id = dev["id"].as_str().unwrap_or_default().to_string();
                let name = dev["name"].as_str().unwrap_or_default().to_string();
                let is_active = dev["is_active"].as_bool().unwrap_or(false);
                let is_restricted = dev["is_restricted"].as_bool().unwrap_or(false);
                let device_type = dev["type"].as_str().unwrap_or("Speaker").to_string();
                let volume_percent = dev["volume_percent"].as_u64().map(|v| v as u32);

                devices.push(DeviceItem {
                    id,
                    is_active,
                    is_restricted,
                    name,
                    device_type,
                    volume_percent,
                });
            }
        }

        Ok(devices)
    }

    /// Transfers playback to the target device.
    pub fn transfer_playback(&self, device_id: &str, play: bool) -> Result<(), String> {
        let body = serde_json::json!({
            "device_ids": [device_id],
            "play": play
        });

        let req = self.auth_put("/me/player")?.json(&body);
        let res = req.send().map_err(|e| format!("Transfer error: {}", e))?;
        if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err("Transfer playback failed".to_string())
        }
    }

    /// Fetches user's saved tracks (Liked Songs).
    pub fn get_saved_tracks(&self, limit: u32) -> Result<Vec<TrackItem>, String> {
        let path = format!("/me/tracks?limit={}", limit);
        let req = self.auth_get(&path)?;
        let res = req
            .send()
            .map_err(|e| format!("Saved tracks error: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Get saved tracks failed: {}", res.status()));
        }

        let val: serde_json::Value = res
            .json()
            .map_err(|e| format!("Saved tracks JSON error: {}", e))?;

        let mut tracks = Vec::new();
        if let Some(items) = val["items"].as_array() {
            for item in items {
                if let Some(track_obj) = item.get("track") {
                    if let Some(track) = parse_spotify_track_json(track_obj) {
                        tracks.push(track);
                    }
                }
            }
        }
        Ok(tracks)
    }

    /// Fetches featured playlists from Spotify browse API.
    pub fn get_featured_playlists(&self) -> Result<Vec<PlaylistItem>, String> {
        let req = self.auth_get("/browse/featured-playlists?limit=20")?;
        let res = req
            .send()
            .map_err(|e| format!("Featured playlists error: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Featured playlists failed: {}", res.status()));
        }

        let val: serde_json::Value = res
            .json()
            .map_err(|e| format!("Playlists JSON error: {}", e))?;

        let mut playlists = Vec::new();
        if let Some(items) = val["playlists"]["items"].as_array() {
            for item in items {
                if item.is_null() {
                    continue;
                }
                let id = item["id"].as_str().unwrap_or_default().to_string();
                let name = item["name"].as_str().unwrap_or_default().to_string();
                let uri = item["uri"].as_str().unwrap_or_default().to_string();
                let description = item["description"].as_str().unwrap_or_default().to_string();
                let owner_name = item["owner"]["display_name"]
                    .as_str()
                    .unwrap_or("Spotify")
                    .to_string();
                let total_tracks = item["tracks"]["total"].as_u64().unwrap_or(0) as u32;

                playlists.push(PlaylistItem {
                    id,
                    name,
                    description,
                    owner_name,
                    total_tracks,
                    uri,
                });
            }
        }

        Ok(playlists)
    }
}

/// Helper to parse a Spotify Track JSON value into `TrackItem`.
fn parse_spotify_track_json(item: &serde_json::Value) -> Option<TrackItem> {
    let id = item["id"].as_str()?.to_string();
    let name = item["name"].as_str()?.to_string();
    let uri = item["uri"].as_str().unwrap_or_default().to_string();
    let duration_ms = item["duration_ms"].as_u64().unwrap_or(0);
    let preview_url = item["preview_url"].as_str().map(|s| s.to_string());
    let popularity = item["popularity"].as_u64().map(|p| p as u32);
    let is_playable = item["is_playable"].as_bool().unwrap_or(true);

    let album_name = item["album"]["name"]
        .as_str()
        .unwrap_or("Single")
        .to_string();

    let mut artists = Vec::new();
    if let Some(artists_arr) = item["artists"].as_array() {
        for a in artists_arr {
            if let Some(a_name) = a["name"].as_str() {
                artists.push(a_name.to_string());
            }
        }
    }

    Some(TrackItem {
        id: id.clone(),
        name,
        artists,
        album_name,
        duration_ms,
        uri: uri.clone(),
        preview_url,
        popularity,
        is_playable,
        source: AudioSource::Spotify { uri, id },
    })
}

/// Simple URL query encoder.
fn urlencoding_simple(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

/// Formats duration in milliseconds as MM:SS.
pub fn format_duration_ms(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

/// Helper function to truncate strings safely to a specific terminal column width.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut current_w = 0;
    let mut out = String::new();
    for c in s.chars() {
        let cw = UnicodeWidthChar::width(c).unwrap_or(0);
        if current_w + cw > max_width {
            break;
        }
        out.push(c);
        current_w += cw;
    }
    out
}

/// Active modal prompt in the music player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerPrompt {
    None,
    Search,
    TokenInput,
    ClientIdInput,
    ClientSecretInput,
    SeekInput,
    VolumeInput,
}

///
/// Represents a music player capable of interacting with Spotify's API, managing playback state,
/// handling search functionality, managing playlists/queue, supporting interactive prompts,
/// and simulating a visualizer.
///
/// # Fields
///
/// - `client: SpotifyClient`
///   The client instance used to communicate with the Spotify API.
///
/// - `playback: PlaybackState`
///   The current playback state, including information about the currently playing track.
///
/// - `active_tab: PlayerTab`
///   Tracks the currently active user interface tab in the music player (e.g., playlist, search).
///
/// - `selected_index: usize`
///   Represents the index of the currently selected item (e.g., track, playlist) within the active tab.
///
/// - `scroll_offset: usize`
///   Tracks the scrolling offset used for rendering content in the UI when the list exceeds the viewable area.
///
/// ## Search State
///
/// - `search_query: String`
///   The current search query input by the user.
///
/// - `search_category: SearchCategory`
///   Specifies the category of the search (e.g., tracks, albums, artists).
///
/// - `search_results: SearchResults`
///   Holds the results of the current search, categorized based on the search type.
///
/// ## Queue & Playlists
///
/// - `queue: Vec<TrackItem>`
///   A vector containing the current playback queue of tracks.
///
/// - `playlists: Vec<PlaylistItem>`
///   A vector of playlists available to the user.
///
/// - `devices: Vec<DeviceItem>`
///   A vector containing devices linked to the Spotify account compatible with playback.
///
/// ## Interactive Prompts
///
/// - `active_prompt: PlayerPrompt`
///   Represents the active user interaction prompt, when applicable (e.g., adding to a playlist).
///
/// - `prompt_input: String`
///   Captures user input for the currently active prompt.
///
/// - `status_message: Option<String>`
///   Displays an optional status update message, used for providing feedback or updating the state.
///
/// ## Visualizer Simulation State
///
/// - `visualizer_tick: usize`
///   Tracks the tick count of the visualizer used for rendering simulated visualizations.
///
/// - `last_tick_time: Instant`
///   Represents the time at which the last visualizer tick was processed.
///
/// This structure ties together the core state and functionality needed for an interactive music player,
/// managing both user-facing and internal data processing operations.
///
#[derive(Debug, Clone)]
pub struct MusicPlayer {
    pub client: SpotifyClient,
    pub playback: PlaybackState,
    pub active_tab: PlayerTab,
    pub selected_index: usize,
    pub scroll_offset: usize,

    // Search state
    pub search_query: String,
    pub search_category: SearchCategory,
    pub search_results: SearchResults,

    // Queue & Playlists
    pub queue: Vec<TrackItem>,
    pub playlists: Vec<PlaylistItem>,
    pub devices: Vec<DeviceItem>,

    // Interactive Prompts
    pub active_prompt: PlayerPrompt,
    pub prompt_input: String,
    pub status_message: Option<String>,

    // Visualizer simulation state
    pub visualizer_tick: usize,
    pub last_tick_time: Instant,
}

impl Default for MusicPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicPlayer {
    #[must_use]
    pub fn new() -> Self {
        let client = SpotifyClient::new();
        Self {
            client,
            playback: PlaybackState::default(),
            active_tab: PlayerTab::NowPlaying,
            selected_index: 0,
            scroll_offset: 0,
            search_query: String::new(),
            search_category: SearchCategory::Tracks,
            search_results: SearchResults::default(),
            queue: Vec::new(),
            playlists: Vec::new(),
            devices: Vec::new(),
            active_prompt: PlayerPrompt::None,
            prompt_input: String::new(),
            status_message: Some(
                "QWX Music Player ready. Press [Tab] to switch views.".to_string(),
            ),
            visualizer_tick: 0,
            last_tick_time: Instant::now(),
        }
    }

    /// Sets status message.
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    /// Refreshes playback status and devices from Spotify API.
    pub fn refresh_playback_state(&mut self) {
        match self.client.get_playback_state() {
            Ok(Some(state)) => {
                self.playback = state;
            }
            Ok(None) => {
                self.playback.is_playing = false;
            }
            Err(e) => {
                // If token missing, inform user gently
                if !self.client.credentials.is_configured() {
                    self.set_status("Spotify not connected. Press [t] to configure token.");
                } else {
                    self.set_status(format!("Spotify sync: {}", e));
                }
            }
        }
    }

    /// Refreshes Spotify Connect devices.
    pub fn refresh_devices(&mut self) {
        match self.client.get_devices() {
            Ok(devs) => {
                self.devices = devs;
                self.set_status(format!("Found {} playback devices.", self.devices.len()));
            }
            Err(e) => {
                self.set_status(format!("Devices error: {}", e));
            }
        }
    }

    /// Refreshes featured playlists / user playlists.
    pub fn refresh_playlists(&mut self) {
        match self.client.get_featured_playlists() {
            Ok(list) => {
                self.playlists = list;
                self.set_status(format!("Loaded {} playlists.", self.playlists.len()));
            }
            Err(e) => {
                self.set_status(format!("Playlists error: {}", e));
            }
        }
    }

    /// Executes the current search query.
    pub fn perform_search(&mut self) {
        if self.search_query.trim().is_empty() {
            return;
        }
        self.set_status(format!("Searching Spotify for '{}'...", self.search_query));
        match self.client.search(&self.search_query, 10) {
            Ok(results) => {
                let count = results.tracks.len() + results.albums.len() + results.playlists.len();
                self.search_results = results;
                self.selected_index = 0;
                self.scroll_offset = 0;
                self.set_status(format!(
                    "Found {} results for '{}'.",
                    count, self.search_query
                ));
            }
            Err(e) => {
                self.set_status(format!("Search failed: {}", e));
            }
        }
    }

    /// Toggle play/pause.
    pub fn toggle_play_pause(&mut self) {
        if self.playback.is_playing {
            match self.client.pause() {
                Ok(_) => {
                    if let Some(synced_at) = self.playback.last_synced_at {
                        self.playback.progress_ms = self
                            .playback
                            .progress_ms
                            .saturating_add(synced_at.elapsed().as_millis() as u64);
                    }
                    self.playback.is_playing = false;
                    self.playback.last_synced_at = Some(Instant::now());
                    self.set_status("Playback paused.");
                }
                Err(e) => self.set_status(format!("Pause failed: {}", e)),
            }
        } else {
            match self.client.resume() {
                Ok(_) => {
                    self.playback.is_playing = true;
                    self.playback.last_synced_at = Some(Instant::now());
                    self.set_status("Playback resumed.");
                }
                Err(e) => {
                    // If resume fails, and we have a selected track in search or queue, play it
                    if let Some(track) = self.queue.first().cloned() {
                        self.play_track(&track);
                    } else {
                        self.set_status(format!("Resume: {}", e));
                    }
                }
            }
        }
    }

    /// Plays a specific track.
    pub fn play_track(&mut self, track: &TrackItem) {
        self.set_status(format!("Playing '{}'...", track.name));
        match &track.source {
            AudioSource::Spotify { uri, .. } => match self.client.play_uri(uri, None) {
                Ok(_) => {
                    self.playback.is_playing = true;
                    self.playback.item = Some(track.clone());
                    self.playback.progress_ms = 0;
                    self.playback.last_synced_at = Some(Instant::now());
                    self.set_status(format!(
                        "▶ Now Playing: {} - {}",
                        track.name,
                        track.artists_str()
                    ));
                }
                Err(e) => {
                    self.set_status(format!("Play error: {}", e));
                }
            },
            AudioSource::LocalFile { path } => {
                self.set_status(format!(
                    "Local playback for {:?} will be supported soon.",
                    path.file_name().unwrap_or_default()
                ));
            }
        }
    }

    /// Plays a playlist or album context URI.
    pub fn play_context(&mut self, uri: &str, name: &str) {
        self.set_status(format!("Playing '{}'...", name));
        match self.client.play_uri(uri, None) {
            Ok(_) => {
                self.playback.is_playing = true;
                self.set_status(format!("▶ Playing: {}", name));
            }
            Err(e) => self.set_status(format!("Play context error: {}", e)),
        }
    }

    /// Skips to the next track.
    pub fn next_track(&mut self) {
        match self.client.next() {
            Ok(_) => {
                self.refresh_playback_state();
                self.set_status("⏭ Skipped to next track.");
            }
            Err(e) => self.set_status(format!("Next track error: {}", e)),
        }
    }

    /// Skips to previous track.
    pub fn prev_track(&mut self) {
        match self.client.previous() {
            Ok(_) => {
                self.refresh_playback_state();
                self.set_status("⏮ Skipped to previous track.");
            }
            Err(e) => self.set_status(format!("Previous track error: {}", e)),
        }
    }

    /// Adjusts volume by delta (+/- percent).
    pub fn adjust_volume(&mut self, delta: i32) {
        let current = self.playback.volume_percent as i32;
        let new_vol = (current + delta).clamp(0, 100) as u32;
        self.playback.volume_percent = new_vol;
        let _ = self.client.set_volume(new_vol);
        self.set_status(format!("Volume: {}%", new_vol));
    }

    /// Toggles shuffle mode.
    pub fn toggle_shuffle(&mut self) {
        let new_state = !self.playback.shuffle_state;
        self.playback.shuffle_state = new_state;
        match self.client.set_shuffle(new_state) {
            Ok(_) => {
                self.set_status(format!("Shuffle: {}", if new_state { "ON" } else { "OFF" }));
            }
            Err(e) => self.set_status(format!("Shuffle error: {}", e)),
        }
    }

    /// Cycles repeat mode.
    pub fn cycle_repeat(&mut self) {
        let next_mode = self.playback.repeat_state.next();
        self.playback.repeat_state = next_mode;
        match self.client.set_repeat(next_mode) {
            Ok(_) => {
                self.set_status(format!("Repeat: {:?}", next_mode));
            }
            Err(e) => self.set_status(format!("Repeat error: {}", e)),
        }
    }

    /// Adds a track to the play queue.
    pub fn add_to_queue(&mut self, track: TrackItem) {
        let name = track.name.clone();
        self.queue.push(track);
        self.set_status(format!(
            "Added '{}' to queue ({} items).",
            name,
            self.queue.len()
        ));
    }

    /// Cycles through active tabs.
    pub fn next_tab(&mut self) {
        let tabs = PlayerTab::all();
        let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.active_tab = tabs[(idx + 1) % tabs.len()];
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.on_tab_switched();
    }
    /// Switches the active tab to the previous tab in the available list of tabs.
    ///
    /// This function cycles through all available `PlayerTab` entries in reverse order.
    /// If the currently active tab is the first in the list, it wraps around to the last tab.
    /// Upon switching the tab, the following adjustments are made:
    /// - Sets the `selected_index` to 0.
    /// - Resets the `scroll_offset` to 0.
    /// - Triggers the `on_tab_switched` callback to handle any actions or updates required
    ///   upon tab change.
    ///
    /// # Assumptions
    /// - The `PlayerTab::all()` function returns a list of all available `PlayerTab` items
    ///   in a consistent and deterministic order.
    /// - The `active_tab` field represents the tab currently active in the player.
    ///
    /// # Panics
    /// - If `PlayerTab::all()` returns an empty list (unlikely scenario), this function
    ///   may panic due to accessing an index that doesn't exist.
    ///
    pub fn prev_tab(&mut self) {
        let tabs = PlayerTab::all();
        let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        self.active_tab = tabs[(idx + tabs.len() - 1) % tabs.len()];
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.on_tab_switched();
    }

    pub fn set_tab(&mut self, tab: PlayerTab) {
        self.active_tab = tab;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.on_tab_switched();
    }

    fn on_tab_switched(&mut self) {
        match self.active_tab {
            PlayerTab::NowPlaying => self.refresh_playback_state(),
            PlayerTab::Devices => self.refresh_devices(),
            PlayerTab::Playlists => {
                if self.playlists.is_empty() {
                    self.refresh_playlists();
                }
            }
            _ => {}
        }
    }

    /// Moves cursor down in the list.
    pub fn move_down(&mut self) {
        let count = self.current_tab_item_count();
        if count > 0 && self.selected_index + 1 < count {
            self.selected_index += 1;
        }
    }

    /// Moves cursor up in the list.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Returns item count in the current tab view.
    pub fn current_tab_item_count(&self) -> usize {
        match self.active_tab {
            PlayerTab::NowPlaying => 0,
            PlayerTab::Search => match self.search_category {
                SearchCategory::Tracks => self.search_results.tracks.len(),
                SearchCategory::Albums => self.search_results.albums.len(),
                SearchCategory::Playlists => self.search_results.playlists.len(),
                SearchCategory::Artists => 0,
            },
            PlayerTab::Queue => self.queue.len(),
            PlayerTab::Playlists => self.playlists.len(),
            PlayerTab::Devices => self.devices.len(),
            PlayerTab::Config => 4,
        }
    }

    /// Handles activation / Enter on the current selected list item.
    pub fn handle_enter_selection(&mut self) {
        match self.active_tab {
            PlayerTab::Search => match self.search_category {
                SearchCategory::Tracks => {
                    if let Some(track) =
                        self.search_results.tracks.get(self.selected_index).cloned()
                    {
                        self.play_track(&track);
                    }
                }
                SearchCategory::Albums => {
                    if let Some(album) =
                        self.search_results.albums.get(self.selected_index).cloned()
                    {
                        self.play_context(&album.uri, &album.name);
                    }
                }
                SearchCategory::Playlists => {
                    if let Some(pl) = self
                        .search_results
                        .playlists
                        .get(self.selected_index)
                        .cloned()
                    {
                        self.play_context(&pl.uri, &pl.name);
                    }
                }
                SearchCategory::Artists => {}
            },
            PlayerTab::Queue => {
                if let Some(track) = self.queue.get(self.selected_index).cloned() {
                    self.play_track(&track);
                }
            }
            PlayerTab::Playlists => {
                if let Some(pl) = self.playlists.get(self.selected_index).cloned() {
                    self.play_context(&pl.uri, &pl.name);
                }
            }
            PlayerTab::Devices => {
                if let Some(dev) = self.devices.get(self.selected_index).cloned() {
                    self.set_status(format!("Transferring playback to '{}'...", dev.name));
                    match self.client.transfer_playback(&dev.id, true) {
                        Ok(_) => {
                            self.set_status(format!("Active device set to '{}'.", dev.name));
                            self.refresh_devices();
                        }
                        Err(e) => self.set_status(format!("Transfer error: {}", e)),
                    }
                }
            }
            PlayerTab::Config => match self.selected_index {
                0 => {
                    self.active_prompt = PlayerPrompt::TokenInput;
                    self.prompt_input.clear();
                }
                1 => {
                    self.active_prompt = PlayerPrompt::ClientIdInput;
                    self.prompt_input.clear();
                }
                2 => {
                    self.active_prompt = PlayerPrompt::ClientSecretInput;
                    self.prompt_input.clear();
                }
                3 => match self.client.request_client_credentials_token() {
                    Ok(tok) => self.set_status(format!(
                        "Token retrieved! ({}...)",
                        &tok[..10.min(tok.len())]
                    )),
                    Err(e) => self.set_status(format!("Auth Error: {}", e)),
                },
                _ => {}
            },
            PlayerTab::NowPlaying => {
                self.toggle_play_pause();
            }
        }
    }

    /// Seeks playback relative to the current position by delta_sec (+/-).
    pub fn seek_relative(&mut self, delta_sec: i64) {
        let current_ms = self.playback.progress_ms as i64;
        let delta_ms = delta_sec * 1000;
        let target_ms = (current_ms + delta_ms).max(0) as u64;
        let total_ms = self
            .playback
            .item
            .as_ref()
            .map(|t| t.duration_ms)
            .unwrap_or(u64::MAX);
        let final_ms = target_ms.min(total_ms);
        self.playback.progress_ms = final_ms;
        self.playback.last_synced_at = Some(Instant::now());
        let _ = self.client.seek(final_ms);
        self.set_status(format!("Seek: {}", format_duration_ms(final_ms)));
    }

    /// Cycles through search categories (Tracks -> Albums -> Playlists -> Artists).
    pub fn cycle_search_category(&mut self) {
        self.search_category = match self.search_category {
            SearchCategory::Tracks => SearchCategory::Albums,
            SearchCategory::Albums => SearchCategory::Playlists,
            SearchCategory::Playlists => SearchCategory::Artists,
            SearchCategory::Artists => SearchCategory::Tracks,
        };
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.set_status(format!("Search category: {}", self.search_category.name()));
    }

    /// Handles a key input event. Returns false if the player mode should exit.
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        // If an interactive prompt is active
        if self.active_prompt != PlayerPrompt::None {
            match code {
                KeyCode::Esc => {
                    self.active_prompt = PlayerPrompt::None;
                    self.prompt_input.clear();
                    self.set_status("Input cancelled.");
                }
                KeyCode::Enter => {
                    self.submit_prompt();
                }
                KeyCode::Backspace => {
                    self.prompt_input.pop();
                }
                KeyCode::Char(c) => {
                    if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT {
                        self.prompt_input.push(c);
                    }
                }
                _ => {}
            }
            return true;
        }

        // Global commands when no prompt is active
        match (modifiers, code) {
            // Exit player
            (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Char('q')) => {
                return false;
            }

            // Tabs navigation
            (KeyModifiers::NONE, KeyCode::Tab) => {
                self.next_tab();
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab) | (KeyModifiers::SHIFT, KeyCode::Tab) => {
                self.prev_tab();
            }
            (KeyModifiers::NONE, KeyCode::Char('1')) => self.set_tab(PlayerTab::NowPlaying),
            (KeyModifiers::NONE, KeyCode::Char('2')) => self.set_tab(PlayerTab::Search),
            (KeyModifiers::NONE, KeyCode::Char('3')) => self.set_tab(PlayerTab::Queue),
            (KeyModifiers::NONE, KeyCode::Char('4')) => self.set_tab(PlayerTab::Playlists),
            (KeyModifiers::NONE, KeyCode::Char('5')) => self.set_tab(PlayerTab::Devices),
            (KeyModifiers::NONE, KeyCode::Char('6')) => self.set_tab(PlayerTab::Config),

            // Playback controls
            (KeyModifiers::NONE, KeyCode::Char(' ')) => {
                self.toggle_play_pause();
            }
            (KeyModifiers::NONE, KeyCode::Char('n'))
            | (KeyModifiers::NONE, KeyCode::Char('>'))
            | (KeyModifiers::NONE, KeyCode::Media(crossterm::event::MediaKeyCode::TrackNext)) => {
                self.next_track();
            }
            (KeyModifiers::NONE, KeyCode::Char('p'))
            | (KeyModifiers::NONE, KeyCode::Char('<'))
            | (KeyModifiers::NONE, KeyCode::Media(crossterm::event::MediaKeyCode::TrackPrevious)) =>
            {
                self.prev_track();
            }
            (KeyModifiers::NONE, KeyCode::Char('+'))
            | (KeyModifiers::NONE, KeyCode::Char('='))
            | (KeyModifiers::NONE, KeyCode::Char(']')) => {
                self.adjust_volume(5);
            }
            (KeyModifiers::NONE, KeyCode::Char('-'))
            | (KeyModifiers::NONE, KeyCode::Char('_'))
            | (KeyModifiers::NONE, KeyCode::Char('[')) => {
                self.adjust_volume(-5);
            }
            (KeyModifiers::NONE, KeyCode::Right) => {
                if self.active_tab == PlayerTab::NowPlaying {
                    self.seek_relative(5);
                } else {
                    self.next_tab();
                }
            }
            (KeyModifiers::NONE, KeyCode::Left) => {
                if self.active_tab == PlayerTab::NowPlaying {
                    self.seek_relative(-5);
                } else {
                    self.prev_tab();
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('z')) | (KeyModifiers::NONE, KeyCode::Char('s')) => {
                self.toggle_shuffle();
            }
            (KeyModifiers::NONE, KeyCode::Char('r')) => {
                self.cycle_repeat();
            }
            (KeyModifiers::SHIFT, KeyCode::Char('R')) | (KeyModifiers::NONE, KeyCode::F(5)) => {
                self.refresh_playback_state();
                self.refresh_devices();
                self.refresh_playlists();
                self.set_status("Player state refreshed.");
            }

            // List Navigation
            (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::NONE, KeyCode::Char('j')) => {
                self.move_down();
            }
            (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::NONE, KeyCode::Char('k')) => {
                self.move_up();
            }
            (KeyModifiers::NONE, KeyCode::PageDown) => {
                for _ in 0..5 {
                    self.move_down();
                }
            }
            (KeyModifiers::NONE, KeyCode::PageUp) => {
                for _ in 0..5 {
                    self.move_up();
                }
            }
            (KeyModifiers::NONE, KeyCode::Home) | (KeyModifiers::NONE, KeyCode::Char('g')) => {
                self.selected_index = 0;
            }
            (KeyModifiers::NONE, KeyCode::End) | (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                let count = self.current_tab_item_count();
                if count > 0 {
                    self.selected_index = count - 1;
                }
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.handle_enter_selection();
            }

            // Interactive prompts and category actions
            (KeyModifiers::NONE, KeyCode::Char('/')) => {
                self.active_prompt = PlayerPrompt::Search;
                self.prompt_input.clear();
                if self.active_tab != PlayerTab::Search {
                    self.active_tab = PlayerTab::Search;
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('c')) if self.active_tab == PlayerTab::Search => {
                self.cycle_search_category();
            }
            (KeyModifiers::NONE, KeyCode::Char('a')) => {
                if self.active_tab == PlayerTab::Search
                    && self.search_category == SearchCategory::Tracks
                {
                    if let Some(track) =
                        self.search_results.tracks.get(self.selected_index).cloned()
                    {
                        self.add_to_queue(track);
                    }
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('d')) | (KeyModifiers::NONE, KeyCode::Delete) => {
                if self.active_tab == PlayerTab::Queue && !self.queue.is_empty() {
                    let removed = self.queue.remove(self.selected_index);
                    if self.selected_index >= self.queue.len() && self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    self.set_status(format!("Removed '{}' from queue.", removed.name));
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('t')) => {
                self.active_prompt = PlayerPrompt::TokenInput;
                self.prompt_input.clear();
            }
            (KeyModifiers::NONE, KeyCode::Char('v')) => {
                self.active_prompt = PlayerPrompt::VolumeInput;
                self.prompt_input.clear();
            }
            (KeyModifiers::NONE, KeyCode::Char('f')) => {
                self.active_prompt = PlayerPrompt::SeekInput;
                self.prompt_input.clear();
            }

            _ => {}
        }
        true
    }

    /// Submits the current active prompt input.
    pub fn submit_prompt(&mut self) {
        let input = self.prompt_input.trim().to_string();
        let prompt = self.active_prompt.clone();
        self.active_prompt = PlayerPrompt::None;
        self.prompt_input.clear();

        match prompt {
            PlayerPrompt::Search => {
                if !input.is_empty() {
                    self.search_query = input;
                    self.active_tab = PlayerTab::Search;
                    self.perform_search();
                }
            }
            PlayerPrompt::TokenInput => {
                if !input.is_empty() {
                    self.client.set_token(&input);
                    self.client.credentials.access_token = Some(input);
                    let _ = self.client.credentials.save_to_config();
                    self.refresh_playback_state();
                    self.refresh_devices();
                    self.set_status("Spotify token updated.");
                }
            }
            PlayerPrompt::ClientIdInput => {
                if !input.is_empty() {
                    self.client.credentials.client_id = Some(input);
                    let _ = self.client.credentials.save_to_config();
                    self.set_status("Spotify Client ID updated.");
                }
            }
            PlayerPrompt::ClientSecretInput => {
                if !input.is_empty() {
                    self.client.credentials.client_secret = Some(input);
                    let _ = self.client.credentials.save_to_config();
                    self.set_status("Spotify Client Secret updated.");
                }
            }
            PlayerPrompt::SeekInput => {
                if let Ok(sec) = input.parse::<u64>() {
                    let pos_ms = sec * 1000;
                    self.playback.progress_ms = pos_ms;
                    self.playback.last_synced_at = Some(Instant::now());
                    let _ = self.client.seek(pos_ms);
                    self.set_status(format!("Seeked to {}s.", sec));
                } else {
                    self.set_status("Invalid seek position.");
                }
            }
            PlayerPrompt::VolumeInput => {
                if let Ok(vol) = input.parse::<u32>() {
                    let vol = vol.min(100);
                    self.playback.volume_percent = vol;
                    let _ = self.client.set_volume(vol);
                    self.set_status(format!("Volume set to {}%.", vol));
                } else {
                    self.set_status("Invalid volume value.");
                }
            }
            PlayerPrompt::None => {}
        }
    }

    /// Renders the complete Music Player TUI to a crossterm writer.
    pub fn draw_player<W: Write>(&mut self, writer: &mut W, w: u16, h: u16) -> io::Result<()> {
        let max_width: u16 = 100;
        let effective_width = w.min(max_width) as usize;
        let offset_x = w.saturating_sub(max_width) / 2;
        execute!(writer, Hide)?;
        let w_usize = w as usize;
        let h_usize = h as usize;
        // Theme colors
        let bg_header = Color::Black;
        let fg_normal = Color::White;
        let fg_muted = Color::Rgb {
            r: 110,
            g: 120,
            b: 145,
        };
        let fg_accent = Color::Rgb {
            r: 30,
            g: 215,
            b: 96,
        }; // Spotify Green
        let fg_gold = Color::Rgb {
            r: 255,
            g: 205,
            b: 85,
        };

        // 1. Header & Player Title
        queue!(
            writer,
            MoveTo(offset_x, 0),
            SetBackgroundColor(bg_header),
            SetForegroundColor(fg_accent)
        )?;

        let title_left = " PLAYER ";
        let dev_name = self
            .playback
            .device
            .as_ref()
            .map(|d| d.name.as_str())
            .unwrap_or("No Device Connected");
        let dev_badge = format!(" [Device: {dev_name}] ");
        let space_top = effective_width.saturating_sub(title_left.width() + dev_badge.width());

        queue!(
            writer,
            Print(title_left),
            SetForegroundColor(fg_muted),
            Print(" ".repeat(space_top)),
            SetForegroundColor(if self.playback.device.is_some() {
                fg_accent
            } else {
                fg_muted
            }),
            Print(dev_badge),
            ResetColor
        )?;
        // 2. Tabs Bar
        queue!(
            writer,
            MoveTo(0, 1),
            SetBackgroundColor(bg_header),
            SetForegroundColor(fg_normal)
        )?;

        let mut tab_line = String::new();
        for tab in PlayerTab::all() {
            let is_sel = *tab == self.active_tab;
            let tab_badge = if is_sel {
                format!("▶ [{}] {} ◀  ", tab.shortcut(), tab.title())
            } else {
                format!(" [{}] {}   ", tab.shortcut(), tab.title())
            };
            tab_line.push_str(&tab_badge);
        }

        let tab_line = tab_line.trim_end().to_string();
        let tab_width = tab_line.width();

        let padded_tabs = if tab_width < w_usize {
            let left_pad = (w_usize - tab_width) / 2;
            let right_pad = w_usize - tab_width - left_pad;
            format!(
                "{}{}{}",
                " ".repeat(left_pad),
                tab_line,
                " ".repeat(right_pad)
            )
        } else {
            truncate_to_width(&tab_line, w_usize)
        };
        queue!(writer, Print(padded_tabs), ResetColor)?;
        // 3. Separator Line
        queue!(
            writer,
            MoveTo(0, 2),
            SetForegroundColor(Color::Rgb {
                r: 40,
                g: 50,
                b: 70
            })
        )?;
        queue!(writer, Print("─".repeat(w_usize)), ResetColor)?;

        // 4. Content Area
        let content_y = 3;
        let content_height = h_usize.saturating_sub(6);

        match self.active_tab {
            PlayerTab::NowPlaying => {
                self.draw_now_playing_tab(writer, content_y, content_height, w_usize)?;
            }
            PlayerTab::Search => {
                self.draw_search_tab(writer, content_y, content_height, w_usize)?;
            }
            PlayerTab::Queue => {
                self.draw_queue_tab(writer, content_y, content_height, w_usize)?;
            }
            PlayerTab::Playlists => {
                self.draw_playlists_tab(writer, content_y, content_height, w_usize)?;
            }
            PlayerTab::Devices => {
                self.draw_devices_tab(writer, content_y, content_height, w_usize)?;
            }
            PlayerTab::Config => {
                self.draw_config_tab(writer, content_y, content_height, w_usize)?;
            }
        }

        // 5. Playback Control Mini Bar (above status line)
        let mini_bar_y = h_usize.saturating_sub(3) as u16;
        queue!(
            writer,
            MoveTo(0, mini_bar_y),
            SetBackgroundColor(Color::Rgb {
                r: 15,
                g: 20,
                b: 30
            }),
            SetForegroundColor(fg_normal)
        )?;

        let play_icon = if self.playback.is_playing {
            " PAUSE"
        } else {
            " PLAY"
        };
        let track_title = self
            .playback
            .item
            .as_ref()
            .map(|t| format!("{} - {}", t.name, t.artists_str()))
            .unwrap_or_else(|| "No track active".to_string());

        let progress_str = if let Some(ref t) = self.playback.item {
            format!(
                "{} / {}",
                format_duration_ms(self.playback.progress_ms),
                format_duration_ms(t.duration_ms)
            )
        } else {
            "--:-- / --:--".to_string()
        };

        let shuffle_ind = if self.playback.shuffle_state {
            "s ON"
        } else {
            "s OFF"
        };
        let repeat_ind = format!("r {:?}", self.playback.repeat_state);
        let vol_ind = format!("v {}%", self.playback.volume_percent);

        let bar_content = format!(
            " [{}] {} │ ⏱ {} │ {} │ {} │ {} ",
            play_icon, track_title, progress_str, shuffle_ind, repeat_ind, vol_ind
        );

        let padded_bar = if bar_content.width() < w_usize {
            format!(
                "{}{}",
                bar_content,
                " ".repeat(w_usize - bar_content.width())
            )
        } else {
            truncate_to_width(&bar_content, w_usize)
        };
        queue!(writer, Print(padded_bar), ResetColor)?;

        // 6. Interactive Prompts or Status Message
        let prompt_y = h_usize.saturating_sub(2) as u16;
        if self.active_prompt != PlayerPrompt::None {
            let prompt_label = match self.active_prompt {
                PlayerPrompt::Search => " Search Spotify: ",
                PlayerPrompt::TokenInput => " Spotify OAuth Bearer Token: ",
                PlayerPrompt::ClientIdInput => " Spotify Client ID: ",
                PlayerPrompt::ClientSecretInput => " Spotify Client Secret: ",
                PlayerPrompt::SeekInput => " Seek to Seconds: ",
                PlayerPrompt::VolumeInput => " Set Volume (0..100): ",
                PlayerPrompt::None => "",
            };

            queue!(
                writer,
                MoveTo(0, prompt_y),
                SetBackgroundColor(Color::Rgb {
                    r: 35,
                    g: 45,
                    b: 70
                }),
                SetForegroundColor(fg_gold),
                Print(prompt_label),
                SetForegroundColor(Color::White),
                Print(&self.prompt_input),
                Print("█"),
                Print(" ".repeat(
                    w_usize.saturating_sub(prompt_label.width() + self.prompt_input.width() + 1)
                )),
                ResetColor
            )?;
        } else {
            let msg = self.status_message.as_deref().unwrap_or("Ready.");
            let status_line = format!(" ℹ {}", msg);
            queue!(
                writer,
                MoveTo(0, prompt_y),
                SetBackgroundColor(bg_header),
                SetForegroundColor(fg_muted)
            )?;
            let padded_status = if status_line.width() < w_usize {
                format!(
                    "{}{}",
                    status_line,
                    " ".repeat(w_usize - status_line.width())
                )
            } else {
                truncate_to_width(&status_line, w_usize)
            };
            queue!(writer, Print(padded_status), ResetColor)?;
        }
        // 7. Footer / Keybindings Quick Help
        let footer_y = h_usize.saturating_sub(1) as u16;
        queue!(
            writer,
            MoveTo(0, footer_y),
            SetBackgroundColor(Color::Rgb {
                r: 12,
                g: 16,
                b: 24
            }),
            SetForegroundColor(fg_muted)
        )?;

        let help_text = "[Space] Play/Pause │ [n] Next │ [p] Prev │ [/] Search │ [a] Queue │ [v] Vol │ [r] Repeat │ [z] Shuffle │ [Tab] Tab │ [q] Quit";
        let help_width = help_text.width();

        let padded_footer = if help_width < w_usize {
            let left_pad = (w_usize - help_width) / 2;
            let right_pad = w_usize - help_width - left_pad;
            format!(
                "{}{}{}",
                " ".repeat(left_pad),
                help_text,
                " ".repeat(right_pad)
            )
        } else {
            truncate_to_width(help_text, w_usize)
        };
        queue!(writer, Print(padded_footer), ResetColor)?;
        writer.flush()
    }
    fn draw_now_playing_tab<W: Write>(
        &self,
        writer: &mut W,
        start_y: usize,
        height: usize,
        w: usize,
    ) -> io::Result<()> {
        let bg_main = Color::Black;
        let fg_muted = Color::White;
        let fg_white = Color::White;

        let (bar_str, bar_width) = if let Some(ref t) = self.playback.item {
            let total = t.duration_ms.max(1);

            let mut actual_prog = self.playback.progress_ms;
            if self.playback.is_playing {
                if let Some(synced_at) = self.playback.last_synced_at {
                    actual_prog =
                        actual_prog.saturating_add(synced_at.elapsed().as_millis() as u64);
                }
            }
            let prog = actual_prog.min(total);
            let pct = (prog * 100) / total;

            let bar_len = 100;
            let filled = ((pct as usize) * bar_len) / 100;
            let empty = bar_len.saturating_sub(filled);

            let s = format!(
                "{} [{}{}] {} ({}%)",
                format_duration_ms(prog),
                "█".repeat(filled),
                " ".repeat(empty),
                format_duration_ms(total),
                pct
            );
            (s.clone(), s.chars().count())
        } else {
            let s =
                "[░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] --:-- / --:--".to_string();
            (s.clone(), s.chars().count())
        };

        let offset_x = w.saturating_sub(bar_width) / 2;

        for y in 0..height {
            queue!(
                writer,
                MoveTo(0, (start_y + y) as u16),
                SetBackgroundColor(bg_main),
                Print(" ".repeat(w))
            )?;

            // Se placer au bon offset pour écrire le contenu
            queue!(
                writer,
                MoveTo(offset_x as u16, (start_y + y) as u16),
                SetForegroundColor(fg_white)
            )?;

            match y {
                2 => {
                    let track_name = self
                        .playback
                        .item
                        .as_ref()
                        .map(|t| t.name.as_str())
                        .unwrap_or("No Active Track");
                    let text = format!(
                        "Track : {}",
                        truncate_to_width(track_name, bar_width.saturating_sub(8))
                    );
                    queue!(writer, Print(text))?;
                }
                3 => {
                    let artists = self
                        .playback
                        .item
                        .as_ref()
                        .map(|t| t.artists_str())
                        .unwrap_or_else(|| "---".to_string());
                    let text = format!(
                        "Artist: {}",
                        truncate_to_width(&artists, bar_width.saturating_sub(8))
                    );
                    queue!(writer, Print(text))?;
                }
                4 => {
                    let album = self
                        .playback
                        .item
                        .as_ref()
                        .map(|t| t.album_name.as_str())
                        .unwrap_or("---");
                    let text = format!(
                        "Album : {}",
                        truncate_to_width(album, bar_width.saturating_sub(8))
                    );
                    queue!(writer, Print(text))?;
                }
                5 => {
                    // 5. Playback Control Mini Bar (above status line)
                    let mini_bar_y = height.saturating_sub(3) as u16;

                    if self.active_tab != PlayerTab::NowPlaying {
                        queue!(
                            writer,
                            MoveTo(0, mini_bar_y),
                            SetBackgroundColor(Color::Rgb {
                                r: 15,
                                g: 20,
                                b: 30
                            }),
                            SetForegroundColor(Color::Reset)
                        )?;

                        let play_icon = if self.playback.is_playing {
                            " PAUSE"
                        } else {
                            " PLAY"
                        };
                        let track_title = self
                            .playback
                            .item
                            .as_ref()
                            .map(|t| format!("{} - {}", t.name, t.artists_str()))
                            .unwrap_or_else(|| "No track active".to_string());

                        let progress_str = if let Some(ref t) = self.playback.item {
                            format!(
                                "{} / {}",
                                format_duration_ms(self.playback.progress_ms),
                                format_duration_ms(t.duration_ms)
                            )
                        } else {
                            "--:-- / --:--".to_string()
                        };

                        let shuffle_ind = if self.playback.shuffle_state {
                            "s ON"
                        } else {
                            "s OFF"
                        };
                        let repeat_ind = format!("r {:?}", self.playback.repeat_state);
                        let vol_ind = format!("v {}%", self.playback.volume_percent);

                        let bar_content = format!(
                            " [{}] {} │ ⏱ {} │ {} │ {} │ {} ",
                            play_icon, track_title, progress_str, shuffle_ind, repeat_ind, vol_ind
                        );

                        let padded_bar = if bar_content.width() < w {
                            format!("{}{}", bar_content, " ".repeat(w - bar_content.width()))
                        } else {
                            truncate_to_width(&bar_content, w)
                        };
                        queue!(writer, Print(padded_bar), ResetColor)?;
                    } else {
                        queue!(
                            writer,
                            MoveTo(0, mini_bar_y),
                            SetBackgroundColor(Color::Black),
                            Print(" ".repeat(w)),
                            ResetColor
                        )?;
                    }
                }
                7 => {
                    queue!(writer, Print(&bar_str))?;
                }
                9 => {
                    let waves = [
                        "♫  ▂ ▃ ▄ ▅ ▆ ▇ █ ▇ ▆ ▅ ▄ ▃ ▂   ▂ ▃ ▄ ▅ ▆ ▇ █ ▇ ▆ ▅ ▄ ▃ ▂   ♫",
                        "♫ ▄ ▆ █ ▇ ▅ ▃ ▂   ▂ ▃ ▅ ▇ █ ▆ ▄ ▃ ▂   ▂ ▃ ▄ ▅ ▆ ▇ █ ▇ ▆ ▅ ▄ ♫",
                        "♫ █ ▇ ▆ ▅ ▄ ▃ ▂   ▂ ▃ ▄ ▅ ▆ ▇ █ ▇ ▆ ▅ ▄ ▃ ▂   ▂ ▃ ▄ ▅ ▆ ▇ █ ♫",
                    ];
                    let wave_line = if self.playback.is_playing {
                        waves[(self.visualizer_tick / 2) % waves.len()]
                    } else {
                        "─ ─ ─ ─ ─ ─ ─ ─ [ PAUSED / IDLE ] ─ ─ ─ ─ ─ ─ ─ ─ ─"
                    };

                    let wave_offset = bar_width.saturating_sub(wave_line.chars().count()) / 2;
                    queue!(
                        writer,
                        MoveTo((offset_x + wave_offset) as u16, (start_y + y) as u16),
                        Print(wave_line)
                    )?;
                }
                11 => {
                    let shuffle_ind = if self.playback.shuffle_state {
                        "Shuffle: ON"
                    } else {
                        "Shuffle: OFF"
                    };
                    let repeat_ind = format!("Repeat: {:?}", self.playback.repeat_state);
                    let vol_ind = format!("Vol: {}%", self.playback.volume_percent);
                    let state_str = format!("{}  │  {}  │  {}", vol_ind, shuffle_ind, repeat_ind);

                    let state_offset = bar_width.saturating_sub(state_str.chars().count()) / 2;
                    queue!(
                        writer,
                        MoveTo((offset_x + state_offset) as u16, (start_y + y) as u16),
                        SetForegroundColor(fg_muted),
                        Print(state_str)
                    )?;
                }
                _ => {}
            }
            queue!(writer, ResetColor)?;
        }
        Ok(())
    }

    fn draw_search_tab<W: Write>(
        &self,
        writer: &mut W,
        start_y: usize,
        height: usize,
        w: usize,
    ) -> io::Result<()> {
        let bg_main = Color::Rgb {
            r: 10,
            g: 14,
            b: 22,
        };
        let bg_sel = Color::Rgb {
            r: 35,
            g: 45,
            b: 70,
        };
        let fg_muted = Color::Rgb {
            r: 110,
            g: 120,
            b: 145,
        };
        let fg_normal = Color::Rgb {
            r: 230,
            g: 235,
            b: 245,
        };

        // Sub-header with category selection
        queue!(
            writer,
            MoveTo(0, start_y as u16),
            SetBackgroundColor(bg_main),
            SetForegroundColor(fg_muted)
        )?;

        let mut cat_line = format!("  Query: '{}' │ Filter: ", self.search_query);
        for cat in SearchCategory::all() {
            let is_sel = *cat == self.search_category;
            if is_sel {
                cat_line.push_str(&format!(" [▶ {}] ", cat.name()));
            } else {
                cat_line.push_str(&format!(" {} ", cat.name()));
            }
        }
        cat_line.push_str(" │ [/] New Search │ [c] Switch Filter");
        let padded_cat = if cat_line.width() < w {
            format!("{}{}", cat_line, " ".repeat(w - cat_line.width()))
        } else {
            truncate_to_width(&cat_line, w)
        };
        queue!(writer, Print(padded_cat))?;

        // Result table header
        queue!(
            writer,
            MoveTo(0, (start_y + 1) as u16),
            SetBackgroundColor(Color::Rgb {
                r: 18,
                g: 24,
                b: 38
            }),
            SetForegroundColor(fg_muted)
        )?;
        let table_header = match self.search_category {
            SearchCategory::Tracks => {
                "    # │ TRACK TITLE                                │ ARTIST                 │ ALBUM                  │ DURATION "
            }
            SearchCategory::Albums => {
                "    # │ ALBUM NAME                                 │ ARTIST                 │ RELEASE DATE │ TRACKS    "
            }
            SearchCategory::Playlists => {
                "    # │ PLAYLIST NAME                              │ OWNER                  │ TOTAL TRACKS            "
            }
            SearchCategory::Artists => {
                "    # │ ARTIST NAME                                │ GENRES                 │ POPULARITY              "
            }
        };
        let padded_th = if table_header.width() < w {
            format!("{}{}", table_header, " ".repeat(w - table_header.width()))
        } else {
            truncate_to_width(table_header, w)
        };
        queue!(writer, Print(padded_th))?;

        // Rows
        let list_y = start_y + 2;
        let list_height = height.saturating_sub(2);

        for y in 0..list_height {
            let item_idx = self.scroll_offset + y;
            let screen_y = (list_y + y) as u16;
            queue!(writer, MoveTo(0, screen_y))?;

            match self.search_category {
                SearchCategory::Tracks => {
                    if let Some(track) = self.search_results.tracks.get(item_idx) {
                        let is_sel = item_idx == self.selected_index;
                        let bg = if is_sel { bg_sel } else { bg_main };
                        let fg = if is_sel { Color::White } else { fg_normal };
                        let prefix = if is_sel { " ▶" } else { "  " };

                        let line = format!(
                            "{} {:>2} │ {:<42} │ {:<22} │ {:<22} │ {:>8}",
                            prefix,
                            item_idx + 1,
                            truncate_to_width(&track.name, 42),
                            truncate_to_width(&track.artists_str(), 22),
                            truncate_to_width(&track.album_name, 22),
                            track.formatted_duration()
                        );
                        let padded_line = if line.width() < w {
                            format!("{}{}", line, " ".repeat(w - line.width()))
                        } else {
                            truncate_to_width(&line, w)
                        };
                        queue!(
                            writer,
                            SetBackgroundColor(bg),
                            SetForegroundColor(fg),
                            Print(padded_line),
                            ResetColor
                        )?;
                    } else {
                        queue!(writer, SetBackgroundColor(bg_main), Print(" ".repeat(w)))?;
                    }
                }
                SearchCategory::Albums => {
                    if let Some(album) = self.search_results.albums.get(item_idx) {
                        let is_sel = item_idx == self.selected_index;
                        let bg = if is_sel { bg_sel } else { bg_main };
                        let prefix = if is_sel { " ▶" } else { "  " };

                        let line = format!(
                            "{} {:>2} │ {:<42} │ {:<22} │ {:<12} │ {:>6} tracks",
                            prefix,
                            item_idx + 1,
                            truncate_to_width(&album.name, 42),
                            truncate_to_width(&album.artists_str(), 22),
                            truncate_to_width(&album.release_date, 12),
                            album.total_tracks
                        );
                        let padded = if line.width() < w {
                            format!("{}{}", line, " ".repeat(w - line.width()))
                        } else {
                            truncate_to_width(&line, w)
                        };
                        queue!(
                            writer,
                            SetBackgroundColor(bg),
                            SetForegroundColor(if is_sel { Color::White } else { fg_normal }),
                            Print(padded),
                            ResetColor
                        )?;
                    } else {
                        queue!(writer, SetBackgroundColor(bg_main), Print(" ".repeat(w)))?;
                    }
                }
                SearchCategory::Playlists => {
                    if let Some(pl) = self.search_results.playlists.get(item_idx) {
                        let is_sel = item_idx == self.selected_index;
                        let bg = if is_sel { bg_sel } else { bg_main };
                        let prefix = if is_sel { " ▶" } else { "  " };

                        let line = format!(
                            "{} {:>2} │ {:<42} │ {:<22} │ {:>6} tracks",
                            prefix,
                            item_idx + 1,
                            truncate_to_width(&pl.name, 42),
                            truncate_to_width(&pl.owner_name, 22),
                            pl.total_tracks
                        );
                        let padded = if line.width() < w {
                            format!("{}{}", line, " ".repeat(w - line.width()))
                        } else {
                            truncate_to_width(&line, w)
                        };
                        queue!(
                            writer,
                            SetBackgroundColor(bg),
                            SetForegroundColor(if is_sel { Color::White } else { fg_normal }),
                            Print(padded),
                            ResetColor
                        )?;
                    } else {
                        queue!(writer, SetBackgroundColor(bg_main), Print(" ".repeat(w)))?;
                    }
                }
                SearchCategory::Artists => {
                    queue!(writer, SetBackgroundColor(bg_main), Print(" ".repeat(w)))?;
                }
            }
        }

        Ok(())
    }

    fn draw_queue_tab<W: Write>(
        &self,
        writer: &mut W,
        start_y: usize,
        height: usize,
        w: usize,
    ) -> io::Result<()> {
        let bg_main = Color::Rgb {
            r: 10,
            g: 14,
            b: 22,
        };
        let bg_sel = Color::Rgb {
            r: 35,
            g: 45,
            b: 70,
        };
        let fg_muted = Color::Rgb {
            r: 110,
            g: 120,
            b: 145,
        };
        let fg_normal = Color::Rgb {
            r: 230,
            g: 235,
            b: 245,
        };

        queue!(
            writer,
            MoveTo(0, start_y as u16),
            SetBackgroundColor(Color::Rgb {
                r: 18,
                g: 24,
                b: 38
            }),
            SetForegroundColor(fg_muted)
        )?;
        let header = format!(
            "  📋 Play Queue ({} items) │ [Enter] Play Track │ [d] Remove",
            self.queue.len()
        );
        queue!(writer, Print(format!("{:<width$}", header, width = w)))?;

        for y in 0..height.saturating_sub(1) {
            let item_idx = self.scroll_offset + y;
            queue!(writer, MoveTo(0, (start_y + 1 + y) as u16))?;

            if let Some(track) = self.queue.get(item_idx) {
                let is_sel = item_idx == self.selected_index;
                let bg = if is_sel { bg_sel } else { bg_main };
                let prefix = if is_sel { " ▶" } else { "  " };

                let line = format!(
                    "{} {:>2} │ {:<40} │ {:<25} │ {}",
                    prefix,
                    item_idx + 1,
                    truncate_to_width(&track.name, 40),
                    truncate_to_width(&track.artists_str(), 25),
                    track.formatted_duration()
                );
                queue!(
                    writer,
                    SetBackgroundColor(bg),
                    SetForegroundColor(if is_sel { Color::White } else { fg_normal }),
                    Print(format!("{:<width$}", line, width = w)),
                    ResetColor
                )?;
            } else {
                queue!(writer, SetBackgroundColor(bg_main), Print(" ".repeat(w)))?;
            }
        }
        Ok(())
    }

    fn draw_playlists_tab<W: Write>(
        &self,
        writer: &mut W,
        start_y: usize,
        height: usize,
        w: usize,
    ) -> io::Result<()> {
        let bg_main = Color::Rgb {
            r: 10,
            g: 14,
            b: 22,
        };
        let bg_sel = Color::Rgb {
            r: 35,
            g: 45,
            b: 70,
        };
        let fg_muted = Color::Rgb {
            r: 110,
            g: 120,
            b: 145,
        };
        let fg_normal = Color::Rgb {
            r: 230,
            g: 235,
            b: 245,
        };

        queue!(
            writer,
            MoveTo(0, start_y as u16),
            SetBackgroundColor(Color::Rgb {
                r: 18,
                g: 24,
                b: 38
            }),
            SetForegroundColor(fg_muted)
        )?;
        let header = "  📁 Playlists & Featured Collections │ [Enter] Play Context │ [r] Reload";
        queue!(writer, Print(format!("{:<width$}", header, width = w)))?;

        for y in 0..height.saturating_sub(1) {
            let item_idx = self.scroll_offset + y;
            queue!(writer, MoveTo(0, (start_y + 1 + y) as u16))?;

            if let Some(pl) = self.playlists.get(item_idx) {
                let is_sel = item_idx == self.selected_index;
                let bg = if is_sel { bg_sel } else { bg_main };
                let prefix = if is_sel { " ▶" } else { "  " };

                let line = format!(
                    "{} {:>2} │ {:<35} │ By {:<20} │ {:>4} tracks │ {}",
                    prefix,
                    item_idx + 1,
                    truncate_to_width(&pl.name, 35),
                    truncate_to_width(&pl.owner_name, 20),
                    pl.total_tracks,
                    truncate_to_width(&pl.description, 30)
                );
                queue!(
                    writer,
                    SetBackgroundColor(bg),
                    SetForegroundColor(if is_sel { Color::White } else { fg_normal }),
                    Print(format!("{:<width$}", line, width = w)),
                    ResetColor
                )?;
            } else {
                queue!(writer, SetBackgroundColor(bg_main), Print(" ".repeat(w)))?;
            }
        }
        Ok(())
    }

    fn draw_devices_tab<W: Write>(
        &self,
        writer: &mut W,
        start_y: usize,
        height: usize,
        w: usize,
    ) -> io::Result<()> {
        let bg_main = Color::Rgb {
            r: 10,
            g: 14,
            b: 22,
        };
        let bg_sel = Color::Rgb {
            r: 35,
            g: 45,
            b: 70,
        };
        let fg_accent = Color::Rgb {
            r: 30,
            g: 215,
            b: 96,
        };
        let fg_normal = Color::Rgb {
            r: 230,
            g: 235,
            b: 245,
        };

        queue!(
            writer,
            MoveTo(0, start_y as u16),
            SetBackgroundColor(Color::Black),
            SetForegroundColor(Color::White),
            SetForegroundColor(Color::Reset),
        )?;
        let header =
            "Available Spotify Connect Devices │ [Enter] Switch Active Device │ [r] Refresh";
        queue!(writer, Print(format!("{:<width$}", header, width = w)))?;

        for y in 0..height.saturating_sub(1) {
            let item_idx = self.scroll_offset + y;
            queue!(writer, MoveTo(0, (start_y + 1 + y) as u16))?;

            if let Some(dev) = self.devices.get(item_idx) {
                let is_sel = item_idx == self.selected_index;
                let bg = if is_sel { bg_sel } else { bg_main };
                let prefix = if is_sel { " ▶" } else { "  " };
                let status_badge = if dev.is_active {
                    " [ACTIVE] "
                } else {
                    "          "
                };
                let vol_str = dev
                    .volume_percent
                    .map(|v| format!("{}%", v))
                    .unwrap_or_else(|| "N/A".to_string());

                let line = format!(
                    "{} {:>2} │ {:<30} │ Type: {:<12} │ Volume: {:<6} │ {}",
                    prefix,
                    item_idx + 1,
                    truncate_to_width(&dev.name, 30),
                    dev.device_type,
                    vol_str,
                    status_badge
                );
                queue!(
                    writer,
                    SetBackgroundColor(bg),
                    SetForegroundColor(if dev.is_active {
                        fg_accent
                    } else if is_sel {
                        Color::White
                    } else {
                        fg_normal
                    }),
                    Print(format!("{:<width$}", line, width = w)),
                    ResetColor
                )?;
            } else {
                queue!(writer, SetBackgroundColor(bg_main), Print(" ".repeat(w)))?;
            }
        }
        Ok(())
    }

    fn draw_config_tab<W: Write>(
        &self,
        writer: &mut W,
        start_y: usize,
        height: usize,
        w: usize,
    ) -> io::Result<()> {
        let bg_main = Color::Rgb {
            r: 10,
            g: 14,
            b: 22,
        };
        let bg_sel = Color::Rgb {
            r: 35,
            g: 45,
            b: 70,
        };
        let fg_muted = Color::Rgb {
            r: 110,
            g: 120,
            b: 145,
        };
        let fg_cyan = Color::Rgb {
            r: 90,
            g: 200,
            b: 250,
        };
        let fg_gold = Color::Rgb {
            r: 255,
            g: 205,
            b: 85,
        };

        let creds = &self.client.credentials;
        let token_masked = creds
            .access_token
            .as_ref()
            .map(|t| {
                format!(
                    "{}...{}",
                    &t[..6.min(t.len())],
                    &t[t.len().saturating_sub(6)..]
                )
            })
            .unwrap_or_else(|| "(Not Set)".to_string());
        let id_val = creds.client_id.as_deref().unwrap_or("(Not Set)");
        let secret_masked = if creds.client_secret.is_some() {
            "********"
        } else {
            "(Not Set)"
        };

        let items = [
            format!("1. Spotify OAuth Bearer Token      : {}", token_masked),
            format!("2. Spotify Client ID                : {}", id_val),
            format!("3. Spotify Client Secret            : {}", secret_masked),
            "4. Request Client Credentials Token  : [Execute]".to_string(),
        ];

        for y in 0..height {
            queue!(writer, MoveTo(0, (start_y + y) as u16))?;

            if y == 1 {
                queue!(
                    writer,
                    SetBackgroundColor(bg_main),
                    SetForegroundColor(fg_gold),
                    Print("   🔑 Spotify Authentication Configuration")
                )?;
            } else if (3..7).contains(&y) {
                let idx = y - 3;
                let is_sel = self.selected_index == idx;
                let bg = if is_sel { bg_sel } else { bg_main };
                let prefix = if is_sel { " ▶ " } else { "   " };
                queue!(
                    writer,
                    SetBackgroundColor(bg),
                    SetForegroundColor(if is_sel { Color::White } else { fg_cyan }),
                    Print(format!(
                        "{:<width$}",
                        format!("{}{}", prefix, items[idx]),
                        width = w
                    )),
                    ResetColor
                )?;
            } else if y == 9 {
                queue!(
                    writer,
                    SetBackgroundColor(bg_main),
                    SetForegroundColor(fg_muted),
                    Print(
                        " Tip: Set env SPOTIFY_TOKEN or ~/.config/qwx/spotify.json to persist credentials."
                    )
                )?;
            } else {
                queue!(writer, SetBackgroundColor(bg_main), Print(" ".repeat(w)))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(0), "00:00");
        assert_eq!(format_duration_ms(65000), "01:05");
        assert_eq!(format_duration_ms(245000), "04:05");
    }

    #[test]
    fn test_repeat_mode_cycle() {
        let mode = RepeatMode::Off;
        let mode = mode.next();
        assert_eq!(mode, RepeatMode::Context);
        let mode = mode.next();
        assert_eq!(mode, RepeatMode::Track);
        let mode = mode.next();
        assert_eq!(mode, RepeatMode::Off);
    }

    #[test]
    fn test_player_tab_navigation() {
        let mut player = MusicPlayer::new();
        assert_eq!(player.active_tab, PlayerTab::NowPlaying);
        player.next_tab();
        assert_eq!(player.active_tab, PlayerTab::Search);
        player.next_tab();
        assert_eq!(player.active_tab, PlayerTab::Queue);
        player.prev_tab();
        assert_eq!(player.active_tab, PlayerTab::Search);
    }

    #[test]
    fn test_queue_management() {
        let mut player = MusicPlayer::new();
        let track = TrackItem {
            id: "123".to_string(),
            name: "Bohemian Rhapsody".to_string(),
            artists: vec!["Queen".to_string()],
            album_name: "A Night at the Opera".to_string(),
            duration_ms: 354000,
            uri: "spotify:track:123".to_string(),
            preview_url: None,
            popularity: Some(90),
            is_playable: true,
            source: AudioSource::Spotify {
                uri: "spotify:track:123".to_string(),
                id: "123".to_string(),
            },
        };

        player.add_to_queue(track.clone());
        assert_eq!(player.queue.len(), 1);
        assert_eq!(player.queue[0].name, "Bohemian Rhapsody");
        assert_eq!(player.queue[0].artists_str(), "Queen");
    }

    #[test]
    fn test_spotify_credentials_init() {
        let creds = SpotifyCredentials::default();
        let _ = creds.is_configured();
    }

    #[test]
    fn test_spotify_credentials_serialization_and_deserialization() {
        let json_data = serde_json::json!({
            "client_id": "test_id",
            "client_secret": "test_secret",
            "access_token": "test_token",
            "refresh_token": "test_refresh"
        });

        let creds: SpotifyCredentials = serde_json::from_value(json_data).unwrap();
        assert_eq!(creds.client_id.as_deref(), Some("test_id"));
        assert_eq!(creds.client_secret.as_deref(), Some("test_secret"));
        assert_eq!(creds.access_token.as_deref(), Some("test_token"));
        assert_eq!(creds.refresh_token.as_deref(), Some("test_refresh"));
        assert!(creds.is_configured());
    }

    #[test]
    fn test_handle_key_playback_and_volume() {
        let mut player = MusicPlayer::new();
        let initial_vol = player.playback.volume_percent;
        // Test volume up with '+'
        player.handle_key(KeyCode::Char('+'), KeyModifiers::NONE);
        assert_eq!(player.playback.volume_percent, (initial_vol + 5).min(100));

        // Test volume down with '-'
        player.handle_key(KeyCode::Char('-'), KeyModifiers::NONE);
        assert_eq!(player.playback.volume_percent, initial_vol);

        // Test repeat cycle with 'r'
        assert_eq!(player.playback.repeat_state, RepeatMode::Off);
        player.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
        assert_eq!(player.playback.repeat_state, RepeatMode::Context);

        // Test shuffle toggle with 'z'
        assert!(!player.playback.shuffle_state);
        player.handle_key(KeyCode::Char('z'), KeyModifiers::NONE);
        assert!(player.playback.shuffle_state);

        // Test tabs shortcuts
        player.handle_key(KeyCode::Char('2'), KeyModifiers::NONE);
        assert_eq!(player.active_tab, PlayerTab::Search);
        player.handle_key(KeyCode::Char('3'), KeyModifiers::NONE);
        assert_eq!(player.active_tab, PlayerTab::Queue);
        player.handle_key(KeyCode::Char('1'), KeyModifiers::NONE);
        assert_eq!(player.active_tab, PlayerTab::NowPlaying);
    }

    #[test]
    fn test_handle_key_prompts_and_exit() {
        let mut player = MusicPlayer::new();
        // Quit with 'q' returns false
        assert!(!player.handle_key(KeyCode::Char('q'), KeyModifiers::NONE));
        // Quit with Esc returns false
        assert!(!player.handle_key(KeyCode::Esc, KeyModifiers::NONE));

        // Trigger search prompt with '/'
        player.handle_key(KeyCode::Char('/'), KeyModifiers::NONE);
        assert_eq!(player.active_prompt, PlayerPrompt::Search);
        assert_eq!(player.active_tab, PlayerTab::Search);

        // Type search query
        player.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
        player.handle_key(KeyCode::Char('o'), KeyModifiers::NONE);
        player.handle_key(KeyCode::Char('c'), KeyModifiers::NONE);
        player.handle_key(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(player.prompt_input, "rock");

        // Backspace
        player.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(player.prompt_input, "roc");

        // Cancel prompt with Esc (returns true, doesn't quit player)
        let stay = player.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(stay);
        assert_eq!(player.active_prompt, PlayerPrompt::None);
        assert_eq!(player.prompt_input, "");

        // Seek prompt
        player.handle_key(KeyCode::Char('f'), KeyModifiers::NONE);
        assert_eq!(player.active_prompt, PlayerPrompt::SeekInput);
        player.handle_key(KeyCode::Char('4'), KeyModifiers::NONE);
        player.handle_key(KeyCode::Char('5'), KeyModifiers::NONE);
        player.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(player.active_prompt, PlayerPrompt::None);
        assert_eq!(player.playback.progress_ms, 45000);
    }
}
