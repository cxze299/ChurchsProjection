pub mod deck;
pub mod discovery;
pub mod pinyin_match;
pub mod quick_play;
pub mod song_parser;

pub use cp_core::*;
pub use deck::{load_deck, Deck, Slide};
pub use discovery::discover;
pub use song_parser::{
    load_songs_from_directories, parse_song_content, parse_song_file, write_song_file, SongFile,
};
