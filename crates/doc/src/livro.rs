//! O livro do Diagrama do Autor Total: dados, partes (pré e pós-textuais), capítulos e formato.
//! É o que a pessoa preenche no painel Livro; `livro.diagramar` (engine) gera as páginas a partir
//! dele. Fica salvo dentro do arquivo `.diagramado`, junto com o documento diagramado.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Limites para dados lidos de arquivo (entrada hostil).
pub const MAX_CAPITULOS: usize = 2000;
pub const MAX_TEXTO: usize = 4_000_000;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Livro {
    pub dados: DadosLivro,
    pub formato: FormatoLivro,
    /// Partes do livro, pela chave de [`PARTES`]: ligada ou não e o texto.
    pub partes: BTreeMap<String, Parte>,
    pub capitulos: Vec<Capitulo>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DadosLivro {
    pub titulo: String,
    pub subtitulo: String,
    pub autor: String,
    pub editora: String,
    pub edicao: String,
    pub cidade: String,
    pub ano: String,
    pub isbn: String,
    pub organizacao: String,
    /// Equipe (revisão, capa, diagramação…): vai na página de créditos, uma pessoa por linha.
    pub equipe: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Parte {
    pub ligada: bool,
    pub texto: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Capitulo {
    pub titulo: String,
    /// Autor do texto (coletâneas): vai logo abaixo do título e no cabeçalho.
    pub autor: String,
    /// Parágrafos separados por quebra de linha. `*itálico*` e `**negrito**` marcam ênfase.
    pub texto: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FormatoLivro {
    /// Chave de [`TAMANHOS`].
    pub tamanho: String,
    /// "editorial" ou "classico".
    pub modelo: String,
    pub fonte: String,
    /// Corpo do texto em pontos; 0 = automático (do tamanho e do modelo).
    pub corpo: f64,
    /// Entrelinha em múltiplos do corpo.
    pub entrelinha: f64,
    /// "normal" ou "ampla".
    pub margem: String,
    pub capitulos_na_direita: bool,
    pub numerar_capitulos: bool,
    pub capitular: bool,
    /// Capitular também no Prefácio, Apresentação, Introdução, Conclusão e Posfácio.
    pub capitular_partes: bool,
    /// Primeiro parágrafo sem recuo no Prefácio, Introdução e demais partes em prosa.
    pub sem_recuo_partes: bool,
    /// Recuo da primeira linha em cm; 0 = automático do modelo.
    pub recuo: f64,
    pub cabecalho: bool,
    /// Coletânea de vários autores: a linha curta logo abaixo do título (um nome) é o autor do
    /// texto, que aparece sob o título e no cabeçalho das páginas da esquerda.
    pub coletanea: bool,
}

impl Default for FormatoLivro {
    fn default() -> Self {
        FormatoLivro {
            tamanho: "14x21".into(),
            modelo: "editorial".into(),
            fonte: "Palatino Linotype".into(),
            corpo: 0.0,
            entrelinha: 1.33,
            margem: "normal".into(),
            capitulos_na_direita: true,
            numerar_capitulos: false,
            capitular: true,
            capitular_partes: false,
            sem_recuo_partes: false,
            recuo: 0.0,
            cabecalho: true,
            coletanea: false,
        }
    }
}

/// Um tamanho de página: chave, nome, largura e altura (cm), margens (cm: cima, baixo, interna,
/// externa) e o corpo automático do texto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tamanho {
    pub chave: &'static str,
    pub nome: &'static str,
    pub w: f64,
    pub h: f64,
    pub m: [f64; 4],
    pub corpo: f64,
}

const PADRAO: Tamanho = Tamanho { chave: "14x21", nome: "14 × 21 cm · romance", w: 14.0, h: 21.0, m: [2.0, 2.3, 2.3, 1.7], corpo: 11.0 };

pub const TAMANHOS: &[Tamanho] = &[
    PADRAO,
    Tamanho { chave: "15.5x23", nome: "15,5 × 23 cm · padrão editorial", w: 15.5, h: 23.0, m: [2.2, 2.5, 2.4, 1.8], corpo: 11.0 },
    Tamanho { chave: "16x23", nome: "16 × 23 cm · ensaio e não ficção", w: 16.0, h: 23.0, m: [2.2, 2.5, 2.5, 1.9], corpo: 11.5 },
    Tamanho { chave: "a5", nome: "A5 · 14,8 × 21 cm", w: 14.8, h: 21.0, m: [2.0, 2.3, 2.3, 1.7], corpo: 11.0 },
    Tamanho { chave: "pocket", nome: "11 × 18 cm · bolso", w: 11.0, h: 18.0, m: [1.6, 1.9, 1.9, 1.4], corpo: 10.0 },
    Tamanho { chave: "a4", nome: "A4 · 21 × 29,7 cm · acadêmico", w: 21.0, h: 29.7, m: [2.5, 2.5, 3.0, 2.0], corpo: 12.0 },
    Tamanho { chave: "kdp5x8", nome: "12,7 × 20,32 cm · Amazon KDP 5×8\"", w: 12.7, h: 20.32, m: [1.9, 2.1, 2.0, 1.4], corpo: 10.5 },
    Tamanho { chave: "kdp55x85", nome: "13,97 × 21,59 cm · Amazon KDP 5,5×8,5\"", w: 13.97, h: 21.59, m: [2.0, 2.3, 2.2, 1.6], corpo: 11.0 },
    Tamanho { chave: "kdp6x9", nome: "15,24 × 22,86 cm · Amazon KDP 6×9\"", w: 15.24, h: 22.86, m: [2.2, 2.5, 2.4, 1.8], corpo: 11.0 },
];

/// O tamanho de `chave` (14 × 21 cm quando não existe).
pub fn tamanho(chave: &str) -> Tamanho {
    TAMANHOS.iter().find(|t| t.chave == chave).copied().unwrap_or(PADRAO)
}

/// Onde a parte fica no livro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lugar {
    /// Antes do texto (pré-textuais), sem número de página impresso nas primeiras.
    Antes,
    Depois,
}

/// Como a parte é composta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jeito {
    /// Página só com o título do livro.
    Falsa,
    /// Folha de rosto, montada dos dados.
    Rosto,
    /// Verso da folha de rosto: direitos, edição, ISBN, ficha.
    Creditos,
    /// Texto curto, embaixo e à direita, em itálico.
    Dedicatoria,
    /// Citação centralizada no terço de baixo.
    Epigrafe,
    /// Sumário gerado com as páginas reais.
    Sumario,
    /// Parte em prosa com título (cada quebra de linha é um parágrafo).
    Prosa,
    /// Texto em que a quebra de linha é só quebra de linha (notas, glossário, referências).
    Linhas,
    /// Sinopse da quarta capa: última página.
    Sinopse,
}

/// As partes, na ordem do livro: chave, nome, lugar e jeito.
pub const PARTES: &[(&str, &str, Lugar, Jeito)] = &[
    ("falsa", "Falsa folha de rosto", Lugar::Antes, Jeito::Falsa),
    ("rosto", "Folha de rosto", Lugar::Antes, Jeito::Rosto),
    ("creditos", "Créditos e ficha catalográfica", Lugar::Antes, Jeito::Creditos),
    ("dedicatoria", "Dedicatória", Lugar::Antes, Jeito::Dedicatoria),
    ("epigrafe", "Epígrafe", Lugar::Antes, Jeito::Epigrafe),
    ("sumario", "Sumário", Lugar::Antes, Jeito::Sumario),
    ("prefacio", "Prefácio", Lugar::Antes, Jeito::Prosa),
    ("apresentacao", "Apresentação", Lugar::Antes, Jeito::Prosa),
    ("introducao", "Introdução", Lugar::Antes, Jeito::Prosa),
    ("conclusao", "Conclusão", Lugar::Depois, Jeito::Prosa),
    ("posfacio", "Posfácio", Lugar::Depois, Jeito::Prosa),
    ("agradecimentos", "Agradecimentos", Lugar::Depois, Jeito::Prosa),
    ("notas", "Notas", Lugar::Depois, Jeito::Linhas),
    ("glossario", "Glossário", Lugar::Depois, Jeito::Linhas),
    ("referencias", "Referências", Lugar::Depois, Jeito::Linhas),
    ("sobre", "Sobre o autor", Lugar::Depois, Jeito::Prosa),
    ("sinopse", "Sinopse (quarta capa)", Lugar::Depois, Jeito::Sinopse),
];

/// Partes que podem abrir com letra capitular (opção do formato).
pub const PARTES_COM_CAPITULAR: &[&str] = &["prefacio", "apresentacao", "introducao", "conclusao", "posfacio"];

impl Livro {
    /// Um livro novo: folha de rosto, créditos e sumário ligados.
    pub fn novo(titulo: &str, autor: &str) -> Livro {
        let mut l = Livro::default();
        l.dados.titulo = titulo.into();
        l.dados.autor = autor.into();
        for k in ["falsa", "rosto", "creditos", "sumario"] {
            l.partes.insert(k.into(), Parte { ligada: true, texto: String::new() });
        }
        l
    }

    pub fn parte(&self, k: &str) -> Option<&Parte> {
        self.partes.get(k)
    }

    /// A parte está ligada (a folha de rosto, sempre).
    pub fn ligada(&self, k: &str) -> bool {
        k == "rosto" || self.partes.get(k).is_some_and(|p| p.ligada)
    }

    /// Corta o que veio grande demais de um arquivo (entrada hostil).
    pub fn limitar(&mut self) {
        self.capitulos.truncate(MAX_CAPITULOS);
        let mut resta = MAX_TEXTO;
        for c in &mut self.capitulos {
            cortar(&mut c.texto, &mut resta);
            cortar(&mut c.titulo, &mut 2000);
        }
        for p in self.partes.values_mut() {
            cortar(&mut p.texto, &mut resta);
        }
    }

    /// Divide um texto em capítulos pelos títulos: linhas marcadas com `# ` ou, quando não há
    /// nenhuma, linhas como "Capítulo 1", "CAPÍTULO I – Nome". Duas linhas de título seguidas, sem
    /// texto entre elas, são um título só (título quebrado no meio).
    pub fn capitulos_do_texto(texto: &str) -> Vec<Capitulo> {
        let linhas: Vec<&str> = texto.lines().collect();
        let tem_marca = linhas.iter().any(|l| l.trim_start().starts_with("# "));
        let e_titulo = |l: &str| -> Option<String> {
            let t = l.trim();
            if tem_marca {
                return t.strip_prefix("# ").map(|s| s.trim().to_string());
            }
            parece_titulo_de_capitulo(t).then(|| t.to_string())
        };
        let mut caps: Vec<Capitulo> = Vec::new();
        for l in linhas {
            if let Some(t) = e_titulo(l) {
                match caps.last_mut() {
                    Some(c) if !c.titulo.is_empty() && c.texto.trim().is_empty() => {
                        c.titulo.push(' ');
                        c.titulo.push_str(&t);
                    }
                    _ => caps.push(Capitulo { titulo: t, ..Default::default() }),
                }
                continue;
            }
            if caps.is_empty() {
                if l.trim().is_empty() {
                    continue;
                }
                caps.push(Capitulo::default());
            }
            if let Some(c) = caps.last_mut() {
                if !c.texto.is_empty() {
                    c.texto.push('\n');
                }
                c.texto.push_str(l.trim_end());
            }
        }
        for c in &mut caps {
            c.texto = c.texto.trim().to_string();
        }
        caps.retain(|c| !c.titulo.is_empty() || !c.texto.is_empty());
        caps
    }
}

impl Livro {
    /// Coletânea: em cada capítulo, a primeira linha curta que parece um nome vira o autor.
    pub fn separar_autores(&mut self) {
        for c in &mut self.capitulos {
            if !c.autor.is_empty() || c.titulo.is_empty() {
                continue;
            }
            let (primeira, resto) = c.texto.split_once('\n').unwrap_or((c.texto.as_str(), ""));
            if parece_nome(primeira) {
                c.autor = primeira.trim().trim_matches('*').trim().to_string();
                c.texto = resto.trim_start().to_string();
            }
        }
    }
}

/// "Nicole Dück", "Maria de Souza Lima": até 6 palavras, com maiúscula, sem pontuação de frase.
pub fn parece_nome(l: &str) -> bool {
    let t = l.trim().trim_matches('*').trim();
    let palavras: Vec<&str> = t.split_whitespace().collect();
    !palavras.is_empty()
        && palavras.len() <= 6
        && t.chars().count() <= 60
        && !t.chars().any(|c| matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | '"' | '“' | '—'))
        && palavras.iter().all(|p| p.chars().next().is_some_and(char::is_uppercase) || matches!(*p, "de" | "da" | "do" | "das" | "dos" | "e"))
        && palavras.iter().filter(|p| p.chars().next().is_some_and(char::is_uppercase)).count() >= 2
}

fn cortar(s: &mut String, resta: &mut usize) {
    if s.len() > *resta {
        let mut i = *resta;
        while i > 0 && !s.is_char_boundary(i) {
            i -= 1;
        }
        s.truncate(i);
    }
    *resta = resta.saturating_sub(s.len());
}

/// "Capítulo 3", "CAPÍTULO III", "Capítulo 12 — A volta", "Parte 2".
pub fn parece_titulo_de_capitulo(t: &str) -> bool {
    if t.is_empty() || t.chars().count() > 120 {
        return false;
    }
    let low = t.to_lowercase();
    for p in ["capítulo ", "capitulo ", "cap. ", "parte "] {
        if let Some(resto) = low.strip_prefix(p) {
            let n: String = resto.chars().take_while(|c| !c.is_whitespace() && !matches!(c, '-' | '–' | '—' | ':' | '.')).collect();
            return !n.is_empty()
                && (n.chars().all(|c| c.is_ascii_digit()) || n.chars().all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c')) || NUMEROS_POR_EXTENSO.contains(&n.as_str()));
        }
    }
    false
}

const NUMEROS_POR_EXTENSO: &[&str] = &[
    "um", "dois", "três", "tres", "quatro", "cinco", "seis", "sete", "oito", "nove", "dez", "onze", "doze", "treze", "catorze", "quatorze", "quinze",
    "dezesseis", "dezessete", "dezoito", "dezenove", "vinte", "primeiro", "segundo", "terceiro", "quarto", "quinto", "sexto", "sétimo", "oitavo",
    "nono", "décimo",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divide_por_marca_e_junta_titulo_quebrado() {
        let t = "Prólogo solto\n# Entendendo a coragem e sua\n\n# aplicação na escrita\nTexto um.\nTexto dois.\n# Outro\nFim.";
        let c = Livro::capitulos_do_texto(t);
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].titulo, "");
        assert_eq!(c[1].titulo, "Entendendo a coragem e sua aplicação na escrita");
        assert_eq!(c[1].texto, "Texto um.\nTexto dois.");
        assert_eq!(c[2].texto, "Fim.");
    }

    #[test]
    fn divide_por_capitulo_sem_marca() {
        let c = Livro::capitulos_do_texto("Capítulo 1\nA.\nCAPÍTULO II — Volta\nB.");
        assert_eq!(c.len(), 2);
        assert_eq!(c[1].titulo, "CAPÍTULO II — Volta");
        assert!(!parece_titulo_de_capitulo("Capítulo de uma história qualquer que não é título"));
    }

    #[test]
    fn limita_texto_em_fronteira_de_caractere() {
        let mut s = "ação".repeat(10);
        let mut r = 2;
        cortar(&mut s, &mut r);
        assert_eq!(s, "a");
    }

    #[test]
    fn coletanea_separa_o_autor() {
        let mut l = Livro::default();
        l.capitulos = Livro::capitulos_do_texto("# Memórias de Sisal\nNicole Dück\nSexta-feira, final de tarde.\n# Outro\nEra uma vez.");
        l.separar_autores();
        assert_eq!(l.capitulos[0].autor, "Nicole Dück");
        assert_eq!(l.capitulos[0].texto, "Sexta-feira, final de tarde.");
        assert_eq!(l.capitulos[1].autor, "");
        assert!(!parece_nome("Era uma vez."));
    }

    #[test]
    fn tamanho_desconhecido_volta_ao_padrao() {
        assert_eq!(tamanho("xyz").chave, "14x21");
    }
}
