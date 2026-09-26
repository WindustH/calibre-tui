use super::{IndexedText, Translator, latin_char, push_search_text};

pub(super) struct JapaneseRomajiTranslator;

impl Translator for JapaneseRomajiTranslator {
  fn index_text(&self, text: &str) -> IndexedText {
    index_japanese_text(text)
  }

  fn normalize_query(&self, query: &str) -> String {
    index_japanese_text(query).into_text()
  }
}

fn index_japanese_text(text: &str) -> IndexedText {
  let chars = text
    .chars()
    .filter(|ch| !ch.is_whitespace())
    .collect::<Vec<_>>();
  let mut indexed = IndexedText::default();
  let mut romaji = String::new();
  let mut previous_vowel = None;

  for (index, &ch) in chars.iter().enumerate() {
    let token_start = indexed.text.len();
    let out = &mut indexed.text;
    let next = chars.get(index + 1).copied();

    if is_sokuon(ch) {
      // っ doubles the next consonant: っか -> kka.
      if let Some(next) = japanese_unit_romaji(&chars, index + 1)
        && let Some(consonant) = first_consonant(&next)
      {
        out.push(consonant);
      }
    } else if is_small_y(ch) && index > 0 && digraph_prefix(chars[index - 1]).is_some() {
      // The small ゃ/ゅ/ょ of a digraph: きゃ -> ky + a.
      out.push_str(small_y_vowel(ch).unwrap_or_default());
    } else if ch == 'ー' {
      // The long vowel mark repeats the previous vowel.
      out.extend(previous_vowel);
    } else if let Some(prefix) = digraph_prefix(ch)
      && next.and_then(small_y_vowel).is_some()
    {
      out.push_str(prefix);
    } else {
      romaji.clear();
      japanese_char(ch, &mut romaji);
      push_search_text(out, ch, &romaji);
    }

    previous_vowel = last_vowel(&indexed.text[token_start..]).or(previous_vowel);
    indexed.token_ends.push(indexed.text.len());
  }

  indexed
}

fn japanese_char(ch: char, out: &mut String) {
  let romaji = match ch {
    'あ' | 'ア' | 'ぁ' | 'ァ' => "a",
    'い' | 'イ' | 'ぃ' | 'ィ' => "i",
    'う' | 'ウ' | 'ぅ' | 'ゥ' => "u",
    'え' | 'エ' | 'ぇ' | 'ェ' => "e",
    'お' | 'オ' | 'ぉ' | 'ォ' => "o",
    'か' | 'カ' => "ka",
    'き' | 'キ' => "ki",
    'く' | 'ク' => "ku",
    'け' | 'ケ' => "ke",
    'こ' | 'コ' => "ko",
    'さ' | 'サ' => "sa",
    'し' | 'シ' => "shi",
    'す' | 'ス' => "su",
    'せ' | 'セ' => "se",
    'そ' | 'ソ' => "so",
    'た' | 'タ' => "ta",
    'ち' | 'チ' => "chi",
    'つ' | 'ツ' => "tsu",
    'て' | 'テ' => "te",
    'と' | 'ト' => "to",
    'な' | 'ナ' => "na",
    'に' | 'ニ' => "ni",
    'ぬ' | 'ヌ' => "nu",
    'ね' | 'ネ' => "ne",
    'の' | 'ノ' => "no",
    'は' | 'ハ' => "ha",
    'ひ' | 'ヒ' => "hi",
    'ふ' | 'フ' => "fu",
    'へ' | 'ヘ' => "he",
    'ほ' | 'ホ' => "ho",
    'ま' | 'マ' => "ma",
    'み' | 'ミ' => "mi",
    'む' | 'ム' => "mu",
    'め' | 'メ' => "me",
    'も' | 'モ' => "mo",
    'や' | 'ヤ' | 'ゃ' | 'ャ' => "ya",
    'ゆ' | 'ユ' | 'ゅ' | 'ュ' => "yu",
    'よ' | 'ヨ' | 'ょ' | 'ョ' => "yo",
    'ら' | 'ラ' => "ra",
    'り' | 'リ' => "ri",
    'る' | 'ル' => "ru",
    'れ' | 'レ' => "re",
    'ろ' | 'ロ' => "ro",
    'わ' | 'ワ' => "wa",
    'を' | 'ヲ' => "wo",
    'ん' | 'ン' => "n",
    'が' | 'ガ' => "ga",
    'ぎ' | 'ギ' => "gi",
    'ぐ' | 'グ' => "gu",
    'げ' | 'ゲ' => "ge",
    'ご' | 'ゴ' => "go",
    'ざ' | 'ザ' => "za",
    'じ' | 'ジ' => "ji",
    'ず' | 'ズ' => "zu",
    'ぜ' | 'ゼ' => "ze",
    'ぞ' | 'ゾ' => "zo",
    'だ' | 'ダ' => "da",
    'ぢ' | 'ヂ' => "ji",
    'づ' | 'ヅ' => "zu",
    'で' | 'デ' => "de",
    'ど' | 'ド' => "do",
    'ば' | 'バ' => "ba",
    'び' | 'ビ' => "bi",
    'ぶ' | 'ブ' => "bu",
    'べ' | 'ベ' => "be",
    'ぼ' | 'ボ' => "bo",
    'ぱ' | 'パ' => "pa",
    'ぴ' | 'ピ' => "pi",
    'ぷ' | 'プ' => "pu",
    'ぺ' | 'ペ' => "pe",
    'ぽ' | 'ポ' => "po",
    'ゔ' | 'ヴ' => "vu",
    _ => return latin_char(ch, out),
  };
  out.push_str(romaji);
}

fn japanese_unit_romaji(chars: &[char], index: usize) -> Option<String> {
  let ch = *chars.get(index)?;
  if let Some(next) = chars.get(index + 1).copied()
    && let (Some(prefix), Some(vowel)) = (digraph_prefix(ch), small_y_vowel(next))
  {
    return Some(format!("{prefix}{vowel}"));
  }
  let mut romaji = String::new();
  japanese_char(ch, &mut romaji);
  Some(romaji)
}

fn digraph_prefix(ch: char) -> Option<&'static str> {
  match ch {
    'き' | 'キ' => Some("ky"),
    'し' | 'シ' => Some("sh"),
    'ち' | 'チ' => Some("ch"),
    'に' | 'ニ' => Some("ny"),
    'ひ' | 'ヒ' => Some("hy"),
    'み' | 'ミ' => Some("my"),
    'り' | 'リ' => Some("ry"),
    'ぎ' | 'ギ' => Some("gy"),
    'じ' | 'ジ' | 'ぢ' | 'ヂ' => Some("j"),
    'び' | 'ビ' => Some("by"),
    'ぴ' | 'ピ' => Some("py"),
    _ => None,
  }
}

fn small_y_vowel(ch: char) -> Option<&'static str> {
  match ch {
    'ゃ' | 'ャ' => Some("a"),
    'ゅ' | 'ュ' => Some("u"),
    'ょ' | 'ョ' => Some("o"),
    _ => None,
  }
}

fn is_small_y(ch: char) -> bool {
  matches!(ch, 'ゃ' | 'ャ' | 'ゅ' | 'ュ' | 'ょ' | 'ョ')
}

fn is_sokuon(ch: char) -> bool {
  matches!(ch, 'っ' | 'ッ')
}

fn first_consonant(text: &str) -> Option<char> {
  text
    .chars()
    .find(|ch| ch.is_ascii_alphabetic() && !matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u'))
}

fn last_vowel(text: &str) -> Option<char> {
  text
    .chars()
    .rev()
    .find(|ch| matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u'))
}
