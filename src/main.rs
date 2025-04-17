use std::{
    env,                   // カレントディレクトリ取得のため
    fs,                    // ファイルシステム操作のため
    io,                    // 標準入出力のため
    path::{Path, PathBuf}, // パス操作のため
};

// id3クレートから必要な機能をインポート
use id3::{Tag, TagLike, Version};
use natord::compare;

fn main() -> io::Result<()> {
    let current_dir = env::current_dir()?;
    let target_dir = current_dir.join("audio_files");

    let mut mp3_files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&target_dir)? {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                // ★デバッグ: 見つかったエントリをすべて表示
                if path.is_file() {
                    if let Some(ext_osstr) = path.extension() {
                        if let Some(ext_str) = ext_osstr.to_str() {
                            if ext_str.eq_ignore_ascii_case("mp3") {
                                mp3_files.push(path);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("  ディレクトリ内のエントリ読み込みエラー: {}", e);
            }
        }
    }

    let total_tracks = mp3_files.len();

    // ★★★ 修正箇所 ★★★
    mp3_files.sort_by(|a, b| {
        // ファイル名を取得 (OsStr) し、比較のために文字列 (String or &str) に変換
        // to_string_lossy は UTF-8 でない場合に代替文字が入るが、ファイル名比較では通常問題ない
        let a_name = a.file_name().unwrap_or_default().to_string_lossy();
        let b_name = b.file_name().unwrap_or_default().to_string_lossy();
        // natord::compare を使って自然順で比較
        compare(&a_name, &b_name)
    });
    // ★★★ 修正箇所ここまで ★★★

    // (以降のトラック番号付与処理は変更なし)
    // ... (set_track_number 関数の呼び出しループ) ...
    println!("トラック番号の書き込みを開始します...");

    for (index, path) in mp3_files.iter().enumerate() {
        let current_track_number = (index + 1) as u32;
        let total_tracks_u32 = total_tracks as u32;

        let file_name_str = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| std::borrow::Cow::from("不明なファイル名"));

        print!(
            "[{}/{}] \"{}\" に トラック番号 {}/{} を書き込み中... ",
            current_track_number,
            total_tracks,
            file_name_str,
            current_track_number,
            total_tracks_u32
        );
        io::Write::flush(&mut io::stdout())?;

        match set_track_number(path, current_track_number, total_tracks_u32) {
            Ok(_) => println!("成功"),
            Err(e) => {
                println!("失敗");
                eprintln!("  エラー: {}", e);
            }
        }
    }

    println!("\n処理が完了しました。");
    Ok(())
}

// set_track_number 関数は変更なし
fn set_track_number(
    path: &Path,
    track: u32,
    total_tracks: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tag = match Tag::read_from_path(path) {
        Ok(tag) => tag,
        Err(id3::Error {
            kind: id3::ErrorKind::NoTag,
            ..
        }) => Tag::new(),
        Err(e) => return Err(format!("タグの読み込みに失敗: {}", e).into()),
    };
    tag.set_track(track);
    tag.set_total_tracks(total_tracks);
    match tag.write_to_path(path, Version::Id3v24) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("タグの書き込みに失敗: {}", e).into()),
    }
}
