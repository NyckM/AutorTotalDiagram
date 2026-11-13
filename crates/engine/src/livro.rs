//! Diagramação automática do livro (Diagrama do Autor Total): a partir de um [`Livro`] (dados,
//! partes e capítulos), monta o documento inteiro: folhas de rosto, créditos, dedicatória,
//! epígrafe, sumário com as páginas reais, partes em prosa, capítulos (cada um começando na página
//! da direita, com capitular), partes finais e a sinopse da quarta capa.
//!
//! O resultado é um documento comum: depois de diagramado, tudo pode ser ajustado à mão.

use std::sync::Arc;

use designcraft_doc::build::NewDocument;
use designcraft_doc::livro::{self, Jeito, Livro, Lugar, PARTES, PARTES_COM_CAPITULAR};
use designcraft_doc::vars::{TextVariable, Use, VarKind, var_char};
use designcraft_doc::{
    Align, CharAttrs, CharFormat, Document, Leading, Margins, ParaAttrs, ParaFormat, ParagraphStyle, SpreadId, SpreadRef, StoryId,
    TabAlign, TabStop, Toc, TocEntry, VerticalJustification, story,
};
use designcraft_geom::Rect;

use crate::Result;
use crate::cmd::bad;

const CM: f64 = 72.0 / 2.54;
const CMD: &str = "livro.diagramar";
/// Máximo de páginas que um capítulo pode ocupar (entrada hostil).
const MAX_PAGINAS_POR_PARTE: usize = 2000;

/// Nomes dos estilos que a diagramação cria (a pessoa pode editá-los depois).
pub mod estilos {
    pub const TEXTO: &str = "Texto";
    pub const TEXTO_ABERTURA: &str = "Texto · abertura com capitular";
    pub const TEXTO_ABERTURA_2: &str = "Texto · abertura com capitular de 2 linhas";
    pub const TEXTO_PRIMEIRO: &str = "Texto · primeiro parágrafo";
    pub const TEXTO_LINHAS: &str = "Texto · linhas";
    pub const TITULO: &str = "Título";
    pub const NUMERO: &str = "Número do capítulo";
    pub const FALSA: &str = "Falsa folha de rosto";
    pub const ROSTO_AUTOR: &str = "Rosto · autor";
    pub const ROSTO_TITULO: &str = "Rosto · título";
    pub const ROSTO_SUBTITULO: &str = "Rosto · subtítulo";
    pub const ROSTO_EDITORA: &str = "Rosto · editora";
    pub const CREDITOS: &str = "Créditos";
    pub const DEDICATORIA: &str = "Dedicatória";
    pub const EPIGRAFE: &str = "Epígrafe";
    pub const EPIGRAFE_AUTOR: &str = "Epígrafe · autor";
    pub const SUMARIO_TITULO: &str = "Sumário · título";
    pub const SUMARIO_ITEM: &str = "Sumário · item";
    pub const CABECALHO: &str = "Cabeçalho";
    pub const FOLIO: &str = "Número da página";
    pub const SINOPSE: &str = "Sinopse";
    pub const SEPARADOR: &str = "Separador de cena";
    pub const AUTOR_TEXTO: &str = "Autor do texto";
    pub const FICHA: &str = "Ficha catalográfica";
}
use estilos as e;

/// Medidas do livro em pontos.
struct Medidas {
    w: f64,
    h: f64,
    margens: Margins,
    corpo: f64,
    entrelinha: f64,
    recuo: f64,
    editorial: bool,
    /// Caracteres que cabem numa linha, mais ou menos.
    cabe: usize,
}

fn medidas(l: &Livro) -> Medidas {
    let f = &l.formato;
    let t = livro::tamanho(&f.tamanho);
    let editorial = f.modelo != "classico";
    let amplas = if f.margem == "ampla" { 0.3 } else { 0.0 };
    let [mut cima, mut baixo, mut dentro, fora] = t.m.map(|m| m + amplas);
    if editorial {
        // Medidas do modelo editorial (14 × 21: 2,2 / 2,1 / 2,1 / 1,7 cm), como na plataforma.
        cima += 0.2;
        baixo -= 0.2;
        dentro -= 0.2;
    }
    let corpo = if f.corpo.is_finite() && f.corpo >= 6.0 && f.corpo <= 24.0 {
        f.corpo
    } else if editorial {
        ((t.corpo - 0.3) * 10.0).round() / 10.0
    } else {
        t.corpo
    };
    let entrelinha = if f.entrelinha.is_finite() && (1.0..=2.5).contains(&f.entrelinha) { f.entrelinha } else { 1.33 };
    let recuo = if f.recuo.is_finite() && f.recuo > 0.0 && f.recuo <= 3.0 { f.recuo * CM } else if editorial { 0.5 * CM } else { 1.2 * corpo };
    Medidas {
        w: t.w * CM,
        h: t.h * CM,
        margens: Margins { top: cima * CM, bottom: baixo * CM, inside: dentro * CM, outside: fora * CM },
        corpo,
        entrelinha,
        recuo,
        editorial,
        cabe: (((t.w - t.m[2] - t.m[3]) * CM) / (corpo * 0.48)).max(10.0) as usize,
    }
}

fn estilo(name: &str, based: Option<&str>, para: ParaAttrs, chars: CharAttrs) -> ParagraphStyle {
    ParagraphStyle { name: name.into(), based_on: based.map(Into::into), next_style: None, para, chars, shortcut: String::new() }
}

fn criar_estilos(d: &mut Document, l: &Livro, m: &Medidas) {
    let pedida = l.formato.fonte.trim();
    // Uma fonte que não está neste computador (Palatino fora do Windows, por exemplo) daria lugar
    // a uma sem serifa: troca pela EB Garamond, que vem com o programa.
    let fonte = if pedida.is_empty() {
        "EB Garamond".to_string()
    } else if designcraft_fonts::FontDb::global().has_family(pedida) {
        pedida.to_string()
    } else if pedida.starts_with("Palatino") && designcraft_fonts::FontDb::global().has_family("Book Antiqua") {
        "Book Antiqua".to_string()
    } else {
        "EB Garamond".to_string()
    };
    let c = m.corpo;
    let lead = (c * m.entrelinha * 100.0).round() / 100.0;
    let letra = |estilo: &str, tam: f64, ent: f64| CharAttrs {
        font_family: Some(fonte.clone()),
        font_style: Some(estilo.into()),
        size: Some(tam),
        leading: Some(Leading::Points(ent)),
        language: Some("Portuguese: Brazilian".into()),
        ..Default::default()
    };
    let largura_util = m.w - m.margens.inside - m.margens.outside;
    let st = d.styles_mut();
    st.paragraph.push(estilo(
        e::TEXTO,
        Some(story::BASIC_PARAGRAPH),
        ParaAttrs {
            align: Some(Align::LeftJustified),
            first_line_indent: Some(m.recuo),
            hyphenate: Some(true),
            hyph_min_word: Some(6),
            hyph_capitalized: Some(false),
            hyph_last_word: Some(false),
            hyph_limit: Some(2),
            keep_first: Some(2),
            keep_last: Some(2),
            ..Default::default()
        },
        letra("Regular", c, lead),
    ));
    st.paragraph.push(estilo(e::TEXTO_PRIMEIRO, Some(e::TEXTO), ParaAttrs { first_line_indent: Some(0.0), ..Default::default() }, CharAttrs::default()));
    st.paragraph.push(estilo(
        e::TEXTO_ABERTURA,
        Some(e::TEXTO),
        ParaAttrs { first_line_indent: Some(0.0), drop_cap_lines: Some(3), drop_cap_chars: Some(1), ..Default::default() },
        CharAttrs::default(),
    ));
    st.paragraph.push(estilo(
        e::TEXTO_ABERTURA_2,
        Some(e::TEXTO_ABERTURA),
        ParaAttrs { drop_cap_lines: Some(2), ..Default::default() },
        CharAttrs::default(),
    ));
    st.paragraph.push(estilo(
        e::TEXTO_LINHAS,
        Some(e::TEXTO),
        ParaAttrs { first_line_indent: Some(0.0), align: Some(Align::Left), space_after: Some(lead * 0.4), ..Default::default() },
        CharAttrs::default(),
    ));
    let tam_titulo = if m.editorial { c * 1.78 } else { c * 1.9 };
    let topo_titulo = if m.editorial { 1.5 * CM } else { (m.h - m.margens.top - m.margens.bottom) * 0.22 };
    st.paragraph.push(estilo(
        e::NUMERO,
        None,
        ParaAttrs {
            align: Some(Align::Center),
            space_before: Some(topo_titulo),
            space_after: Some(c * 0.8),
            hyphenate: Some(false),
            keep_with_next: Some(2),
            ..Default::default()
        },
        CharAttrs { tracking: Some(200.0), capitalization: Some(designcraft_doc::Capitalization::AllCaps), ..letra("Regular", c * 0.75, c) },
    ));
    st.paragraph.push(estilo(
        e::TITULO,
        None,
        ParaAttrs {
            align: Some(if m.editorial { Align::Center } else { Align::Left }),
            space_before: Some(topo_titulo),
            space_after: Some(if m.editorial { 1.6 * CM } else { 2.2 * c }),
            hyphenate: Some(false),
            keep_with_next: Some(3),
            keep_all_lines: Some(true),
            keep_lines_together: Some(true),
            balance_ragged: Some(true),
            ..Default::default()
        },
        letra("Regular", tam_titulo, tam_titulo * 1.3),
    ));
    let centro = |a: ParaAttrs| ParaAttrs { align: Some(Align::Center), hyphenate: Some(false), first_line_indent: Some(0.0), ..a };
    st.paragraph.push(estilo(e::FALSA, None, centro(ParaAttrs::default()), letra("Regular", c * 1.6, c * 2.0)));
    st.paragraph.push(estilo(e::ROSTO_AUTOR, None, centro(ParaAttrs::default()), letra("Regular", c * 0.93, c * 1.4)));
    st.paragraph.push(estilo(e::ROSTO_TITULO, None, centro(ParaAttrs { space_after: Some(c * 0.6), ..Default::default() }), letra("Regular", c * 2.8, c * 3.4)));
    st.paragraph.push(estilo(e::ROSTO_SUBTITULO, None, centro(ParaAttrs::default()), letra("Italic", c * 1.15, c * 1.5)));
    st.paragraph.push(estilo(e::ROSTO_EDITORA, None, centro(ParaAttrs::default()), letra("Regular", c * 0.85, c * 1.35)));
    st.paragraph.push(estilo(
        e::CREDITOS,
        None,
        ParaAttrs { align: Some(Align::Left), space_after: Some(c * 0.55), hyphenate: Some(false), ..Default::default() },
        letra("Regular", c * 0.8, c * 1.25),
    ));
    st.paragraph.push(estilo(e::DEDICATORIA, None, ParaAttrs { align: Some(Align::Right), hyphenate: Some(false), ..Default::default() }, letra("Italic", c, lead)));
    st.paragraph.push(estilo(
        e::EPIGRAFE,
        None,
        ParaAttrs { align: Some(if m.editorial { Align::Center } else { Align::Right }), hyphenate: Some(false), ..Default::default() },
        letra("Italic", c * 1.05, lead),
    ));
    st.paragraph.push(estilo(
        e::EPIGRAFE_AUTOR,
        Some(e::EPIGRAFE),
        ParaAttrs { space_before: Some(c * 1.4), ..Default::default() },
        CharAttrs { font_style: Some("Regular".into()), size: Some(c * 0.93), ..Default::default() },
    ));
    st.paragraph.push(estilo(
        e::SUMARIO_TITULO,
        None,
        ParaAttrs {
            align: Some(if m.editorial { Align::Center } else { Align::Left }),
            space_before: Some(if m.editorial { 1.0 * CM } else { topo_titulo }),
            space_after: Some(1.6 * CM),
            hyphenate: Some(false),
            ..Default::default()
        },
        letra("Regular", c * 2.0, c * 2.5),
    ));
    st.paragraph.push(estilo(
        e::SUMARIO_ITEM,
        None,
        ParaAttrs {
            align: Some(Align::Left),
            hyphenate: Some(false),
            space_after: Some(c * 0.7),
            tabs: Some(vec![TabStop { position: (largura_util - 0.5).max(36.0), align: TabAlign::Right, leader: ".".into(), align_on: String::new() }]),
            ..Default::default()
        },
        letra("Regular", c, lead),
    ));
    st.paragraph.push(estilo(
        e::SEPARADOR,
        Some(e::TEXTO),
        ParaAttrs { align: Some(Align::Center), first_line_indent: Some(0.0), space_before: Some(lead * 0.5), space_after: Some(lead * 0.5), ..Default::default() },
        CharAttrs::default(),
    ));
    st.paragraph.push(estilo(
        e::AUTOR_TEXTO,
        None,
        ParaAttrs {
            align: Some(if m.editorial { Align::Center } else { Align::Left }),
            space_after: Some(if m.editorial { 1.6 * CM } else { 2.2 * c }),
            hyphenate: Some(false),
            keep_with_next: Some(2),
            ..Default::default()
        },
        letra("Regular", c * 0.93, c * 1.3),
    ));
    st.paragraph.push(estilo(
        e::FICHA,
        None,
        ParaAttrs {
            align: Some(Align::Left),
            hyphenate: Some(false),
            border_on: Some(true),
            border_weights: Some([0.5; 4]),
            border_offsets: Some([4.0; 4]),
            border_merge: Some(true),
            ..Default::default()
        },
        CharAttrs {
            font_family: Some("JetBrains Mono".into()),
            font_style: Some("Regular".into()),
            size: Some(6.5),
            leading: Some(Leading::Points(8.5)),
            ..Default::default()
        },
    ));
    st.paragraph.push(estilo(e::CABECALHO, None, centro(ParaAttrs::default()), letra("Regular", c * 0.75, c)));
    st.paragraph.push(estilo(e::FOLIO, None, centro(ParaAttrs::default()), letra("Regular", c * 0.82, c)));
    st.paragraph.push(estilo(
        e::SINOPSE,
        Some(e::TEXTO),
        ParaAttrs { first_line_indent: Some(0.0), space_after: Some(c * 0.8), ..Default::default() },
        CharAttrs::default(),
    ));
}

/// Um parágrafo a compor: estilo e trechos (texto, itálico, negrito).
struct Par {
    estilo: &'static str,
    trechos: Vec<(String, bool, bool)>,
    /// Ajuste só deste parágrafo (fora do estilo).
    ajuste: Option<ParaAttrs>,
}

impl Par {
    /// Texto sem marcas de ênfase (as marcas internas do texto, se vierem de fora, saem).
    fn simples(estilo: &'static str, t: &str) -> Par {
        Par { estilo, trechos: vec![(limpo(t), false, false)], ajuste: None }
    }
    /// Texto com marcas internas de propósito (número da página, variáveis).
    fn cru(estilo: &'static str, t: &str) -> Par {
        Par { estilo, trechos: vec![(t.to_string(), false, false)], ajuste: None }
    }
    fn marcado(estilo: &'static str, t: &str) -> Par {
        Par { estilo, trechos: enfase(t), ajuste: None }
    }
}

fn limpo(t: &str) -> String {
    t.chars().filter(|c| !('\u{E000}'..='\u{F8FF}').contains(c) && (!c.is_control() || *c == '\t' || *c == '\n')).collect()
}

/// `*itálico*` e `**negrito**` em trechos.
fn enfase(t: &str) -> Vec<(String, bool, bool)> {
    let mut out: Vec<(String, bool, bool)> = Vec::new();
    let (mut it, mut ng) = (false, false);
    let mut atual = String::new();
    let mut chars = t.chars().peekable();
    let mut antes = ' ';
    while let Some(ch) = chars.next() {
        // Asterisco entre espaços ("* * *") é texto, não marca de ênfase.
        let solto = ch == '*' && antes.is_whitespace() && chars.peek().is_none_or(|c| c.is_whitespace());
        antes = ch;
        if ch == '*' && !solto {
            let duplo = chars.peek() == Some(&'*');
            if duplo {
                chars.next();
            }
            if !atual.is_empty() {
                out.push((std::mem::take(&mut atual), it, ng));
            }
            if duplo {
                ng = !ng;
            } else {
                it = !it;
            }
            continue;
        }
        // Caracteres de controle do texto (marcas internas) não entram vindos de fora.
        if ('\u{E000}'..='\u{F8FF}').contains(&ch) || (ch.is_control() && ch != '\t') {
            continue;
        }
        atual.push(ch);
    }
    if !atual.is_empty() {
        out.push((atual, it, ng));
    }
    // Um asterisco solto (sem par) volta a ser texto: se o último trecho ficou "aberto" à toa, tudo bem.
    out
}

/// Grava os parágrafos na história `sid` (substitui o texto).
fn escrever(d: &mut Document, sid: StoryId, pars: &[Par]) -> Result<()> {
    let st = d.story_mut(sid).ok_or_else(|| bad(CMD, "história sumiu"))?;
    let mut s = designcraft_doc::Story::new(sid);
    s.frames = std::mem::take(&mut st.frames);
    s.direction = st.direction;
    let mut text = String::new();
    let mut paras = Vec::new();
    let mut runs: Vec<story::CharRun> = Vec::new();
    for (i, p) in pars.iter().enumerate() {
        if i > 0 {
            text.push('\n');
            if let Some(r) = runs.last_mut() {
                r.len += 1;
            } else {
                runs.push(story::CharRun { len: 1, format: CharFormat::default() });
            }
        }
        paras.push(ParaFormat { style: p.estilo.into(), para: p.ajuste.clone().unwrap_or_default(), ..Default::default() });
        for (t, it, ng) in &p.trechos {
            let t: String = t.chars().map(|c| if c == '\n' { story::FORCED_LINE_BREAK } else { c }).collect();
            if t.is_empty() {
                continue;
            }
            text.push_str(&t);
            let mut f = CharFormat::default();
            match (it, ng) {
                (true, true) => f.over.font_style = Some("Bold Italic".into()),
                (true, false) => f.over.font_style = Some("Italic".into()),
                (false, true) => f.over.font_style = Some("Bold".into()),
                _ => {}
            }
            match runs.last_mut() {
                Some(r) if r.format == f => r.len += t.len(),
                _ => runs.push(story::CharRun { len: t.len(), format: f }),
            }
        }
    }
    if paras.is_empty() {
        paras.push(ParaFormat { style: e::TEXTO.into(), ..Default::default() });
    }
    if runs.is_empty() {
        runs.push(story::CharRun { len: 0, format: CharFormat::default() });
    }
    s.text = text;
    s.paras = paras;
    s.chars = runs;
    s.check().map_err(|m| bad(CMD, m))?;
    *st = s;
    st.rev = st.rev.wrapping_add(1);
    Ok(())
}

/// Os parágrafos de um texto: cada quebra de linha é um parágrafo (prosa) e linhas em branco
/// não viram parágrafos vazios.
fn paragrafos(t: &str) -> Vec<String> {
    t.replace("\r\n", "\n").replace('\r', "\n").split('\n').map(str::trim).filter(|p| !p.is_empty()).map(str::to_string).collect()
}

/// O montador: o documento e as páginas-mestre.
struct Montador<'a> {
    d: Document,
    l: &'a Livro,
    m: Medidas,
    /// Páginas-mestre: texto (cabeçalho e número), abertura (só número), pré-textuais (só número).
    texto: SpreadId,
    abertura: SpreadId,
    pre: SpreadId,
    /// Quantas páginas já estão em uso (a primeira página do documento começa vazia).
    usadas: usize,
}

impl Montador<'_> {
    /// Uma página nova no fim, com a página-mestre `mestre`. Devolve o índice.
    fn pagina(&mut self, mestre: Option<SpreadId>) -> Result<usize> {
        if self.usadas == 0 && self.d.page_count() >= 1 {
            self.d.apply_parent(&[0], mestre)?;
            self.usadas = 1;
            return Ok(0);
        }
        let ultima = self.d.page_count().saturating_sub(1);
        self.d.insert_pages(Some(ultima), 1, mestre)?;
        self.usadas = self.d.page_count();
        Ok(self.d.page_count().saturating_sub(1))
    }

    /// A próxima página nova será a da direita (ímpar): se não for, entra uma página em branco.
    fn ir_para_direita(&mut self) -> Result<()> {
        // Páginas espelhadas começando na direita: índice par = direita.
        if self.usadas % 2 == 1 {
            self.pagina(None)?;
        }
        Ok(())
    }

    fn area(&self, pg: usize) -> Result<(usize, Rect)> {
        let (si, pi) = self.d.page_loc(pg).ok_or_else(|| bad(CMD, "página sumiu"))?;
        Ok((si, self.d.spreads[si].pages[pi].margin_rect()))
    }

    /// Uma moldura de texto em `pg`, no retângulo relativo à área útil `(x0,y0,x1,y1)` em frações.
    fn moldura(&mut self, pg: usize, frac: [f64; 4], vj: VerticalJustification, pars: &[Par]) -> Result<StoryId> {
        let (si, r) = self.area(pg)?;
        let rect = Rect::new(
            r.x0 + r.width() * frac[0],
            r.y0 + r.height() * frac[1],
            r.x0 + r.width() * frac[2],
            r.y0 + r.height() * frac[3],
        );
        let layer = self.d.default_layer();
        let (fid, sid) = self.d.add_text_frame(SpreadRef::Doc(si), rect, layer, "", ParaFormat::default())?;
        if let Some(tf) = self.d.item_mut(fid).and_then(|it| it.text_frame_mut()) {
            tf.options.vertical_justification = vj;
        }
        escrever(&mut self.d, sid, pars)?;
        Ok(sid)
    }

    /// Uma parte que corre por quantas páginas precisar: começa numa página nova (na direita, se
    /// pedido), com `primeira` como página-mestre da primeira e `resto` das seguintes.
    fn corrida(&mut self, direita: bool, primeira: Option<SpreadId>, resto: Option<SpreadId>, pars: &[Par]) -> Result<(usize, StoryId)> {
        if direita {
            self.ir_para_direita()?;
        }
        let pg = self.pagina(primeira)?;
        let (si, r) = self.area(pg)?;
        let layer = self.d.default_layer();
        let (_, sid) = self.d.add_text_frame(SpreadRef::Doc(si), r, layer, "", ParaFormat::default())?;
        escrever(&mut self.d, sid, pars)?;
        crate::cmd::autoflow_livro(&mut self.d, sid, MAX_PAGINAS_POR_PARTE, CMD)?;
        let fim = self.d.page_count();
        if fim > pg + 1 {
            let seguintes: Vec<usize> = (pg + 1..fim).collect();
            self.d.apply_parent(&seguintes, resto)?;
        }
        self.usadas = fim;
        Ok((pg, sid))
    }
}

/// O título e os parágrafos de uma parte em prosa ou de um capítulo.
fn corpo(titulo: Option<&str>, autor: Option<&str>, numero: Option<String>, texto: &str, capitular: bool, sem_recuo_primeiro: bool, por_linha: bool, por_linha_cabe: usize) -> Vec<Par> {
    let mut v = Vec::new();
    if let Some(n) = numero {
        v.push(Par::simples(e::NUMERO, &n));
    }
    if let Some(t) = titulo.filter(|t| !t.trim().is_empty()) {
        let mut p = Par::simples(e::TITULO, t.trim());
        if let Some(a) = autor.filter(|a| !a.trim().is_empty()) {
            // Com o autor logo abaixo, a distância até o texto fica no autor.
            p.ajuste = Some(ParaAttrs { space_after: Some(8.0), ..Default::default() });
            v.push(p);
            v.push(Par::simples(e::AUTOR_TEXTO, a.trim()));
        } else {
            v.push(p);
        }
    }
    let mut depois_do_separador = false;
    for (i, p) in paragrafos(texto).iter().enumerate() {
        if !por_linha && e_separador(p) {
            v.push(Par::simples(e::SEPARADOR, p));
            depois_do_separador = true;
            continue;
        }
        let est = if por_linha {
            e::TEXTO_LINHAS
        } else if i == 0 && capitular {
            // A capitular desce 3 linhas: um primeiro parágrafo mais curto que isso a deixaria
            // por cima do parágrafo seguinte.
            let linhas = p.chars().count() / por_linha_cabe.max(1) + 1;
            match linhas {
                0 | 1 => e::TEXTO_PRIMEIRO,
                2 => e::TEXTO_ABERTURA_2,
                _ => e::TEXTO_ABERTURA,
            }
        } else if (i == 0 && sem_recuo_primeiro) || std::mem::take(&mut depois_do_separador) {
            e::TEXTO_PRIMEIRO
        } else {
            e::TEXTO
        };
        v.push(Par::marcado(est, p));
    }
    v
}

/// "* * *", "***", "#", "~": separador de cena.
fn e_separador(p: &str) -> bool {
    let t: String = p.chars().filter(|c| !c.is_whitespace()).collect();
    !t.is_empty() && t.chars().count() <= 5 && t.chars().all(|c| matches!(c, '*' | '#' | '~' | '•' | '⁂' | '❧' | '-' | '—'))
}

/// Monta o documento do livro.
pub fn diagramar(l: &Livro) -> Result<Document> {
    let mut l = l.clone();
    l.limitar();
    if l.formato.coletanea {
        l.separar_autores();
    }
    let m = medidas(&l);
    let titulo_doc = if l.dados.titulo.trim().is_empty() { "Livro sem título".to_string() } else { l.dados.titulo.trim().to_string() };
    let nd = NewDocument {
        title: titulo_doc.clone(),
        width: m.w,
        height: m.h,
        pages: 1,
        start_page: 1,
        facing_pages: true,
        primary_text_frame: false,
        columns: 1,
        gutter: 12.0,
        margins: m.margens,
        units: designcraft_geom::Unit::Millimeters,
        ..Default::default()
    };
    let mut d = Document::new(&nd);
    d.title = titulo_doc.clone();
    criar_estilos(&mut d, &l, &m);

    // Variáveis do cabeçalho: autor (páginas da esquerda) e título do capítulo (da direita).
    let autor = l.dados.autor.trim().to_string();
    d.text_variables.push(TextVariable::new("Autor do livro", VarKind::Custom { text: if autor.is_empty() { titulo_doc.clone() } else { autor.clone() } }));
    let var_autor = var_char(d.text_variables.len().saturating_sub(1));
    if l.formato.coletanea {
        // Coletânea: na esquerda, o autor de cada texto (o último que apareceu).
        if let Some(v) = d.text_variables.last_mut() {
            v.kind = VarKind::RunningHeader { style: e::AUTOR_TEXTO.into(), use_: Use::FirstOnPage, character: false };
        }
    }
    d.text_variables.push(TextVariable::new("Título do capítulo", VarKind::RunningHeader { style: e::TITULO.into(), use_: Use::FirstOnPage, character: false }));
    let var_cap = var_char(d.text_variables.len().saturating_sub(1));

    // Páginas-mestre. A "A" do documento novo vira a de texto.
    let texto = d.parents.first().map(|p| p.id).ok_or_else(|| bad(CMD, "sem página-mestre"))?;
    if let Some(p) = d.parents.first_mut() {
        let p = Arc::make_mut(p);
        if let Some(pi) = p.parent.as_mut() {
            pi.name = "Texto".into();
        }
    }
    let abertura = d.add_parent("B", "Abertura de capítulo", 1, 12.0, m.margens);
    let pre = d.add_parent("C", "Partes antes do texto", 1, 12.0, m.margens);
    let mut mt = Montador { d, l: &l, m, texto, abertura, pre, usadas: 0 };
    mestres(&mut mt, var_autor, var_cap)?;

    let fmt = &l.formato;
    let direita = fmt.capitulos_na_direita;
    let dados = &l.dados;
    let mut sumario: Option<(usize, StoryId)> = None;

    for (k, nome, lugar, jeito) in PARTES {
        if *lugar != Lugar::Antes || !l.ligada(k) {
            continue;
        }
        let texto_parte = l.parte(k).map(|p| p.texto.as_str()).unwrap_or("");
        match jeito {
            Jeito::Falsa => {
                mt.ir_para_direita()?;
                let pg = mt.pagina(None)?;
                mt.moldura(pg, [0.0, 0.18, 1.0, 0.45], VerticalJustification::Center, &[Par::simples(e::FALSA, &titulo_doc)])?;
            }
            Jeito::Rosto => {
                mt.ir_para_direita()?;
                let pg = mt.pagina(None)?;
                let mut topo = Vec::new();
                for linha in dados.autor.lines().filter(|x| !x.trim().is_empty()) {
                    topo.push(Par::simples(e::ROSTO_AUTOR, linha.trim()));
                }
                if !topo.is_empty() {
                    mt.moldura(pg, [0.0, 0.0, 1.0, 0.2], VerticalJustification::Top, &topo)?;
                }
                let mut meio = vec![Par::simples(e::ROSTO_TITULO, &titulo_doc)];
                if !dados.subtitulo.trim().is_empty() {
                    meio.push(Par::simples(e::ROSTO_SUBTITULO, dados.subtitulo.trim()));
                }
                mt.moldura(pg, [0.0, 0.25, 1.0, 0.62], VerticalJustification::Center, &meio)?;
                if !dados.organizacao.trim().is_empty() {
                    let mut org = vec![Par::simples(e::ROSTO_EDITORA, "Organização")];
                    org.extend(dados.organizacao.lines().filter(|x| !x.trim().is_empty()).map(|x| Par::simples(e::ROSTO_AUTOR, x.trim())));
                    mt.moldura(pg, [0.0, 0.64, 1.0, 0.8], VerticalJustification::Top, &org)?;
                }
                let pe: Vec<Par> = [dados.edicao.trim(), dados.editora.trim(), &[dados.cidade.trim(), dados.ano.trim()].iter().filter(|x| !x.is_empty()).copied().collect::<Vec<_>>().join(" · ")]
                    .iter()
                    .filter(|x| !x.is_empty())
                    .map(|x| Par::simples(e::ROSTO_EDITORA, x))
                    .collect();
                if !pe.is_empty() {
                    mt.moldura(pg, [0.0, 0.78, 1.0, 1.0], VerticalJustification::Bottom, &pe)?;
                }
            }
            Jeito::Creditos => {
                // Verso da folha de rosto: página da esquerda logo em seguida.
                if mt.usadas % 2 == 0 {
                    mt.pagina(None)?;
                }
                let pg = mt.pagina(None)?;
                let ano = if dados.ano.trim().is_empty() { String::new() } else { format!(" {}", dados.ano.trim()) };
                let dono = if autor.is_empty() { dados.editora.trim().to_string() } else { autor.clone() };
                let mut pars = vec![Par::simples(e::CREDITOS, &format!("©{ano} {dono}. Todos os direitos reservados."))];
                for x in [&dados.edicao, &dados.editora].iter().filter(|x| !x.trim().is_empty()) {
                    pars.push(Par::simples(e::CREDITOS, x.trim()));
                }
                if !dados.isbn.trim().is_empty() {
                    pars.push(Par::simples(e::CREDITOS, &format!("ISBN {}", dados.isbn.trim())));
                }
                for x in dados.equipe.lines().filter(|x| !x.trim().is_empty()) {
                    pars.push(Par::simples(e::CREDITOS, x.trim()));
                }
                // A ficha catalográfica (o texto desta parte) vai numa caixa, em letra de máquina,
                // como as bibliotecárias entregam.
                for x in texto_parte.lines() {
                    pars.push(Par::simples(e::FICHA, x.trim_end()));
                }
                while pars.last().is_some_and(|p| p.estilo == e::FICHA && p.trechos.iter().all(|t| t.0.trim().is_empty())) {
                    pars.pop();
                }
                mt.moldura(pg, [0.0, 0.3, 1.0, 1.0], VerticalJustification::Bottom, &pars)?;
            }
            Jeito::Dedicatoria if !texto_parte.trim().is_empty() => {
                mt.ir_para_direita()?;
                let pg = mt.pagina(None)?;
                let pars: Vec<Par> = paragrafos(texto_parte).iter().map(|p| Par::marcado(e::DEDICATORIA, p)).collect();
                mt.moldura(pg, [0.35, 0.35, 1.0, 1.0], VerticalJustification::Top, &pars)?;
            }
            Jeito::Epigrafe if !texto_parte.trim().is_empty() => {
                mt.ir_para_direita()?;
                let pg = mt.pagina(None)?;
                let linhas = paragrafos(texto_parte);
                let n = linhas.len();
                let pars: Vec<Par> = linhas
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let autoria = i + 1 == n && n > 1 && (p.starts_with('—') || p.starts_with('-') || p.starts_with('–'));
                        Par::marcado(if autoria { e::EPIGRAFE_AUTOR } else { e::EPIGRAFE }, p)
                    })
                    .collect();
                let x0 = if mt.m.editorial { 0.06 } else { 0.25 };
                mt.moldura(pg, [x0, 0.3, 1.0 - if mt.m.editorial { 0.06 } else { 0.0 }, 1.0], VerticalJustification::Top, &pars)?;
            }
            Jeito::Sumario => {
                // O sumário reserva as páginas agora (com itens de mentira do mesmo tamanho) e é
                // preenchido no fim, com as páginas reais.
                let mut pars = vec![Par::simples(e::SUMARIO_TITULO, nome)];
                for t in titulos_do_sumario(&l) {
                    pars.push(Par::simples(e::SUMARIO_ITEM, &format!("{t}\t000")));
                }
                sumario = Some(mt.corrida(true, None, None, &pars)?);
            }
            Jeito::Prosa => {
                let cap = fmt.capitular_partes && fmt.capitular && PARTES_COM_CAPITULAR.contains(k);
                let pars = corpo(Some(nome), None, None, texto_parte, cap, fmt.sem_recuo_partes || !mt.m.editorial, false, mt.m.cabe);
                if pars.len() > 1 {
                    mt.corrida(direita, Some(mt.pre), Some(mt.pre), &pars)?;
                }
            }
            _ => {}
        }
    }

    // Capítulos.
    let mut n = 0;
    for c in &l.capitulos {
        let titulado = !c.titulo.trim().is_empty();
        if titulado {
            n += 1;
        }
        let numero = (titulado && fmt.numerar_capitulos).then(|| format!("Capítulo {n}"));
        let pars = corpo(Some(&c.titulo), Some(&c.autor), numero, &c.texto, fmt.capitular, true, false, mt.m.cabe);
        if pars.is_empty() {
            continue;
        }
        mt.corrida(direita, Some(mt.abertura), Some(mt.texto), &pars)?;
    }

    // Depois do texto.
    for (k, nome, lugar, jeito) in PARTES {
        if *lugar != Lugar::Depois || !l.ligada(k) {
            continue;
        }
        let texto_parte = l.parte(k).map(|p| p.texto.as_str()).unwrap_or("");
        if texto_parte.trim().is_empty() {
            continue;
        }
        match jeito {
            Jeito::Prosa | Jeito::Linhas => {
                let cap = fmt.capitular_partes && fmt.capitular && PARTES_COM_CAPITULAR.contains(k);
                let pars = corpo(Some(nome), None, None, texto_parte, cap, fmt.sem_recuo_partes || !mt.m.editorial, *jeito == Jeito::Linhas, mt.m.cabe);
                mt.corrida(direita, Some(mt.abertura), Some(mt.texto), &pars)?;
            }
            Jeito::Sinopse => {
                // A quarta capa fica numa página da esquerda, a última.
                if mt.usadas % 2 == 0 {
                    mt.pagina(None)?;
                }
                let pg = mt.pagina(None)?;
                let pars: Vec<Par> = paragrafos(texto_parte).iter().map(|p| Par::marcado(e::SINOPSE, p)).collect();
                mt.moldura(pg, [0.08, 0.12, 0.92, 1.0], VerticalJustification::Top, &pars)?;
            }
            _ => {}
        }
    }

    let mut d = mt.d;
    if let Some((_, sid)) = sumario {
        let toc = Toc {
            story: sid,
            title: "Sumário".into(),
            entries: vec![TocEntry { entry_style: Some(e::SUMARIO_ITEM.into()), ..TocEntry::new(e::TITULO, 1) }],
            page_numbers: true,
            title_style: Some(e::SUMARIO_TITULO.into()),
            create_bookmarks: true,
        };
        crate::cmd::toc_preencher(&mut d, &toc)?;
        d.toc = Some(toc);
    }
    d.livro = Some(Box::new(l));
    Ok(d)
}

/// Os títulos que vão para o sumário (para reservar as páginas dele).
fn titulos_do_sumario(l: &Livro) -> Vec<String> {
    let mut v = Vec::new();
    for (k, nome, lugar, jeito) in PARTES {
        if *lugar == Lugar::Antes && *jeito == Jeito::Prosa && l.ligada(k) && l.parte(k).is_some_and(|p| !p.texto.trim().is_empty()) {
            v.push((*nome).to_string());
        }
    }
    v.extend(l.capitulos.iter().filter(|c| !c.titulo.trim().is_empty()).map(|c| c.titulo.trim().to_string()));
    for (k, nome, lugar, jeito) in PARTES {
        if *lugar == Lugar::Depois && matches!(jeito, Jeito::Prosa | Jeito::Linhas) && l.ligada(k) && l.parte(k).is_some_and(|p| !p.texto.trim().is_empty()) {
            v.push((*nome).to_string());
        }
    }
    v
}

/// Cabeçalho e número da página nas páginas-mestre.
fn mestres(mt: &mut Montador, var_autor: Option<char>, var_cap: Option<char>) -> Result<()> {
    let editorial = mt.m.editorial;
    let corpo = mt.m.corpo;
    let cabecalho = mt.l.formato.cabecalho;
    for (id, com_cabecalho) in [(mt.texto, cabecalho), (mt.abertura, false), (mt.pre, false)] {
        let pi = mt.d.parent_index(id).ok_or_else(|| bad(CMD, "página-mestre sumiu"))?;
        let paginas: Vec<(designcraft_doc::PageSide, Rect, Rect)> =
            mt.d.parents[pi].pages.iter().map(|p| (p.side, p.bounds(), p.margin_rect())).collect();
        let layer = mt.d.default_layer();
        for (lado, b, r) in paginas {
            let esquerda = lado == designcraft_doc::PageSide::Left;
            // Número da página: no pé, centralizado (editorial) ou no canto de fora (clássico).
            let altura = corpo * 1.6;
            let y0 = r.y1 + (b.y1 - r.y1) * 0.35;
            let folio = Rect::new(r.x0, y0, r.x1, y0 + altura);
            let (_, sid) = mt.d.add_text_frame(SpreadRef::Parent(pi), folio, layer, "", ParaFormat::default())?;
            escrever(&mut mt.d, sid, &[Par::cru(e::FOLIO, &story::PAGE_NUMBER.to_string())])?;
            if !editorial {
                let alinhar = if esquerda { Align::Left } else { Align::Right };
                if let Some(st) = mt.d.story_mut(sid) {
                    st.format_paras(0..0, |f| f.para.align = Some(alinhar));
                }
            }
            if com_cabecalho {
                let y1 = r.y0 - (r.y0 - b.y0) * 0.38;
                let cab = Rect::new(r.x0, y1 - corpo * 1.4, r.x1, y1);
                let v = if esquerda { var_autor } else { var_cap };
                if let Some(v) = v {
                    let (fid, sid) = mt.d.add_text_frame(SpreadRef::Parent(pi), cab, layer, "", ParaFormat::default())?;
                    if let Some(tf) = mt.d.item_mut(fid).and_then(|it| it.text_frame_mut()) {
                        tf.options.vertical_justification = VerticalJustification::Bottom;
                    }
                    escrever(&mut mt.d, sid, &[Par::cru(e::CABECALHO, &v.to_string())])?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use designcraft_doc::livro::{Capitulo, Parte};

    fn livro_exemplo() -> Livro {
        let mut l = Livro::novo("Escama", "Nyck Maftum");
        l.dados.subtitulo = "Contos do deserto".into();
        l.dados.editora = "Autor Total".into();
        l.dados.ano = "2026".into();
        l.partes.insert("dedicatoria".into(), Parte { ligada: true, texto: "Para quem atravessa desertos.".into() });
        l.partes.insert("epigrafe".into(), Parte { ligada: true, texto: "Tudo se resume a coragem.\n— Fumitake Koga".into() });
        l.partes.insert("prefacio".into(), Parte { ligada: true, texto: "Um prefácio curto.\nCom dois parágrafos.".into() });
        let par = "Um céu escuro da cor preta chocava com a brancura do chão que ofuscava a vista, dunas de um pó fino que desenhavam longas montanhas na areia. ".repeat(6);
        for i in 0..3 {
            l.capitulos.push(Capitulo { titulo: format!("Capítulo de teste {i}"), texto: format!("{par}\n{par}\n*Itálico* e **negrito**.\n{par}"), ..Default::default() });
        }
        l
    }

    #[test]
    fn diagrama_livro_com_sumario_e_capitulos_na_direita() {
        let d = diagramar(&livro_exemplo()).unwrap();
        assert!(d.page_count() >= 10, "páginas: {}", d.page_count());
        let toc = d.toc.as_ref().unwrap();
        let st = d.story(toc.story).unwrap();
        assert!(st.text.contains("Capítulo de teste 2"), "{}", st.text);
        assert!(!st.text.contains("000"), "o sumário ficou com os números provisórios: {}", st.text);
        // Cada capítulo abre numa página da direita (índice par).
        for (sid, s) in &d.stories {
            let r = s.para_ranges();
            for (i, f) in s.paras.iter().enumerate() {
                if f.style == e::TITULO && s.text[r[i].clone()].starts_with("Capítulo de teste") {
                    let fr = s.frames[0];
                    let pg = d.page_of_item(fr).unwrap();
                    assert_eq!(pg % 2, 0, "capítulo em página da esquerda ({sid:?})");
                }
            }
        }
        assert!(d.livro.is_some());
    }

    #[test]
    fn enfase_marca_italico_e_negrito() {
        assert_eq!(enfase("* * *").iter().map(|x| x.0.as_str()).collect::<String>(), "* * *");
        let t = enfase("a *b* **c** d");
        assert_eq!(t.iter().map(|x| x.0.as_str()).collect::<String>(), "a b c d");
        assert!(t.iter().any(|x| x.0 == "b" && x.1 && !x.2));
        assert!(t.iter().any(|x| x.0 == "c" && !x.1 && x.2));
    }

    #[test]
    fn texto_hostil_nao_quebra() {
        let mut l = Livro::novo("", "");
        l.capitulos.push(Capitulo { titulo: "\u{E000}\u{E100}x".into(), texto: "\u{0}\u{E004}*".into(), ..Default::default() });
        l.formato.corpo = f64::NAN;
        l.formato.entrelinha = f64::INFINITY;
        l.formato.tamanho = "???".into();
        assert!(diagramar(&l).is_ok());
    }
}
