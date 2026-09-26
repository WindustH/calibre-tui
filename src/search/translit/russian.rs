use super::{IndexedText, Translator, index_by_char, latin_char};

pub(super) struct RussianLatinTranslator;

impl Translator for RussianLatinTranslator {
  fn index_text(&self, text: &str) -> IndexedText {
    index_by_char(text, russian_char)
  }

  fn normalize_query(&self, query: &str) -> String {
    index_by_char(query, russian_char).into_text()
  }
}

fn russian_char(ch: char, out: &mut String) {
  let latin = match ch {
    'А' | 'а' => "a",
    'Б' | 'б' => "b",
    'В' | 'в' => "v",
    'Г' | 'г' => "g",
    'Д' | 'д' => "d",
    'Е' | 'е' => "e",
    'Ё' | 'ё' => "yo",
    'Ж' | 'ж' => "zh",
    'З' | 'з' => "z",
    'И' | 'и' => "i",
    'Й' | 'й' => "y",
    'К' | 'к' => "k",
    'Л' | 'л' => "l",
    'М' | 'м' => "m",
    'Н' | 'н' => "n",
    'О' | 'о' => "o",
    'П' | 'п' => "p",
    'Р' | 'р' => "r",
    'С' | 'с' => "s",
    'Т' | 'т' => "t",
    'У' | 'у' => "u",
    'Ф' | 'ф' => "f",
    'Х' | 'х' => "kh",
    'Ц' | 'ц' => "ts",
    'Ч' | 'ч' => "ch",
    'Ш' | 'ш' => "sh",
    'Щ' | 'щ' => "shch",
    'Ъ' | 'ъ' => "",
    'Ы' | 'ы' => "y",
    'Ь' | 'ь' => "",
    'Э' | 'э' => "e",
    'Ю' | 'ю' => "yu",
    'Я' | 'я' => "ya",
    _ => return latin_char(ch, out),
  };
  out.push_str(latin);
}
