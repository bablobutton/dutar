use crate::queue::Metadata;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::read_from_path;
use lofty::tag::Accessor;
use log::error;
use ratatui::crossterm::event::KeyCode;
use std::fs;
use std::path::Path;

pub fn for_each_subdir<F>(dir: &Path, cb: &mut F)
where
    F: FnMut(&fs::DirEntry),
{
    if dir.is_dir() {
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        for_each_subdir(path.as_path(), cb);
                    } else {
                        cb(&entry);
                    }
                }
            }
            Err(error) => {
                error!("Error traversing directories: {error}");
            }
        }
    }
}

pub fn extract_metadata(path: &Path) -> Option<Metadata> {
    // try read fromat from path extention
    let tagged_file = read_from_path(path)
        // if couldn't read from extension, read from contents
        .or_else(|_| Probe::open(path)?.guess_file_type()?.read())
        .inspect_err(|e| {
            error!(
                "could not read metadata from {}, error: {e}",
                path.display()
            )
        })
        .ok()?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    match tag {
        Some(t) => Some(Metadata {
            title: t.title().map(|s| s.to_string()).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string()
            }),
            artist: t
                .artist()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string()),
            album: t
                .album()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string()),
            duration_seconds: tagged_file.properties().duration().as_secs(),
        }),
        None => Some(Metadata {
            title: "".to_string(),
            artist: "".to_string(),
            album: "".to_string(),
            duration_seconds: tagged_file.properties().duration().as_secs(),
        }),
    }
}

pub fn count_digits(num: u64) -> usize {
    if num == 0 {
        return 1;
    }
    (num.ilog10() + 1) as usize
}

pub fn map_key_code(key_code: KeyCode) -> KeyCode {
    match key_code {
        KeyCode::Char(c) => KeyCode::Char(map_russian_to_qwerty(c)),
        other => other,
    }
}

fn map_russian_to_qwerty(c: char) -> char {
    match c {
        'й' => 'q',
        'ц' => 'w',
        'у' => 'e',
        'к' => 'r',
        'е' => 't',
        'н' => 'y',
        'г' => 'u',
        'ш' => 'i',
        'щ' => 'o',
        'з' => 'p',
        'х' => '[',
        'ъ' => ']',
        'ф' => 'a',
        'ы' => 's',
        'в' => 'd',
        'а' => 'f',
        'п' => 'g',
        'р' => 'h',
        'о' => 'j',
        'л' => 'k',
        'д' => 'l',
        'ж' => ';',
        'э' => '\'',
        'я' => 'z',
        'ч' => 'x',
        'с' => 'c',
        'м' => 'v',
        'и' => 'b',
        'т' => 'n',
        'ь' => 'm',
        'б' => ',',
        'ю' => '.',
        'ё' => '`',
        // uppercase
        'Й' => 'Q',
        'Ц' => 'W',
        'У' => 'E',
        'К' => 'R',
        'Е' => 'T',
        'Н' => 'Y',
        'Г' => 'U',
        'Ш' => 'I',
        'Щ' => 'O',
        'З' => 'P',
        'Х' => '{',
        'Ъ' => '}',
        'Ф' => 'A',
        'Ы' => 'S',
        'В' => 'D',
        'А' => 'F',
        'П' => 'G',
        'Р' => 'H',
        'О' => 'J',
        'Л' => 'K',
        'Д' => 'L',
        'Ж' => ':',
        'Э' => '"',
        'Я' => 'Z',
        'Ч' => 'X',
        'С' => 'C',
        'М' => 'V',
        'И' => 'B',
        'Т' => 'N',
        'Ь' => 'M',
        'Б' => '<',
        'Ю' => '>',
        'Ё' => '~',
        other => other,
    }
}
