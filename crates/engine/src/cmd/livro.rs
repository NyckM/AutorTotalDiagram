//! Comandos do livro (Diagrama do Autor Total): criar, ler, guardar, importar o texto e diagramar.

use designcraft_doc::livro::Livro;
use serde_json::{Value, json};

use super::{CommandSpec, bad, cmd, has_doc, str_param};
use crate::{DocState, Result, Session};

/// Maior arquivo de texto aceito na importação.
const MAX_ARQUIVO: u64 = 64 * 1024 * 1024;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "livro.novo",
            "Novo livro…",
            ["File"],
            None,
            "{titulo?, autor?, tamanho?: \"14x21\"|\"15.5x23\"|\"16x23\"|\"a5\"|\"pocket\"|\"a4\"|\"kdp5x8\"|\"kdp55x85\"|\"kdp6x9\", modelo?: \"editorial\"|\"classico\", livro?: Livro} → {index}",
            |_| Ok(()),
            novo
        ),
        cmd!(query "livro.obter", "Livro", [], None, "{} → o livro do documento (dados, formato, partes, capítulos)", has_doc, |s, _| {
            let d = &s.doc()?.doc;
            let l = d.livro.as_deref().cloned().unwrap_or_else(|| Livro::novo(&d.title, ""));
            serde_json::to_value(l).map_err(|e| bad("livro.obter", e.to_string()))
        }),
        cmd!("livro.definir", "Guardar o livro", [], None, "{livro: Livro} — guarda dados, formato, partes e capítulos sem diagramar de novo", has_doc, |s, p| {
            let l = livro_param("livro.definir", p)?.ok_or_else(|| bad("livro.definir", "falta `livro`"))?;
            s.edit(|d, _| {
                d.livro = Some(Box::new(l));
                Ok(json!({}))
            })
        }),
        cmd!(
            "livro.diagramar",
            "Diagramar o livro",
            ["Layout"],
            Some("Ctrl+Shift+D"),
            "{livro?: Livro} — monta todas as páginas a partir do livro (o do documento, ou o dado aqui) → {paginas}",
            has_doc,
            diagramar
        ),
        cmd!(
            "livro.importar_texto",
            "Importar texto do livro…",
            ["File"],
            None,
            "{path: arquivo .docx, .txt, .md ou .rtf, substituir?: true (troca os capítulos; false acrescenta)} → {capitulos}",
            has_doc,
            importar
        ),
    ]
}

fn livro_param(cmd: &str, p: &Value) -> Result<Option<Livro>> {
    match p.get("livro").filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => {
            let mut l: Livro = serde_json::from_value(v.clone()).map_err(|e| bad(cmd, format!("livro inválido: {e}")))?;
            l.limitar();
            Ok(Some(l))
        }
    }
}

fn novo(s: &mut Session, p: &Value) -> Result<Value> {
    let mut l = match livro_param("livro.novo", p)? {
        Some(l) => l,
        None => Livro::novo(str_param(p, "titulo").unwrap_or("").trim(), str_param(p, "autor").unwrap_or("").trim()),
    };
    if let Some(t) = str_param(p, "tamanho") {
        l.formato.tamanho = t.into();
    }
    if let Some(m) = str_param(p, "modelo") {
        l.formato.modelo = m.into();
    }
    let d = crate::livro::diagramar(&l)?;
    let i = s.add_document(DocState::new(d, None));
    Ok(json!({"index": i}))
}

fn diagramar(s: &mut Session, p: &Value) -> Result<Value> {
    let dado = livro_param("livro.diagramar", p)?;
    s.edit(|d, sel| {
        let l = match dado {
            Some(l) => l,
            None => d.livro.as_deref().cloned().unwrap_or_else(|| Livro::novo(&d.title, "")),
        };
        let novo = crate::livro::diagramar(&l)?;
        let paginas = novo.page_count();
        // Mantém o que é do arquivo (informações e ids), troca as páginas.
        let info = std::mem::take(&mut d.file_info);
        *d = novo;
        d.file_info = info;
        *sel = Default::default();
        Ok(json!({"paginas": paginas}))
    })
}

fn importar(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_param(p, "path").ok_or_else(|| bad("livro.importar_texto", "falta `path`"))?.to_string();
    let substituir = p.get("substituir").and_then(Value::as_bool).unwrap_or(true);
    let meta = std::fs::metadata(&path).map_err(|e| bad("livro.importar_texto", format!("não consegui abrir {path}: {e}")))?;
    if meta.len() > MAX_ARQUIVO {
        return Err(bad("livro.importar_texto", "o arquivo é grande demais (mais de 64 MB)"));
    }
    let bytes = std::fs::read(&path).map_err(|e| bad("livro.importar_texto", format!("não consegui ler {path}: {e}")))?;
    let texto = texto_marcado(&path, &bytes)?;
    let caps = Livro::capitulos_do_texto(&texto);
    let n = caps.len();
    s.edit(|d, _| {
        let mut l = d.livro.as_deref().cloned().unwrap_or_else(|| Livro::novo(&d.title, ""));
        if substituir {
            l.capitulos = caps;
        } else {
            l.capitulos.extend(caps);
        }
        if l.formato.coletanea {
            l.separar_autores();
        }
        l.limitar();
        d.livro = Some(Box::new(l));
        Ok(json!({"capitulos": n}))
    })
}

/// O texto de um arquivo com os títulos de nível 1 marcados com `# ` e a ênfase com `*`/`**`.
pub fn texto_marcado(nome: &str, bytes: &[u8]) -> Result<String> {
    let imp = designcraft_textimport::import(nome, bytes).map_err(|e| bad("livro.importar_texto", e.to_string()))?;
    let st = &imp.story;
    let titulo_estilo = |nome: &str| {
        let n = nome.to_lowercase().replace(' ', "");
        n == "heading1" || n == "título1" || n == "titulo1" || n == "title" || n == "título" || n == "titulo" || n == "h1" || n == "capítulo" || n == "capitulo"
    };
    let italico_estilo: Vec<String> = imp
        .char_styles
        .iter()
        .filter(|c| c.chars.font_style.as_deref().is_some_and(|f| f.to_lowercase().contains("italic")))
        .map(|c| c.name.clone())
        .collect();
    let negrito_estilo: Vec<String> =
        imp.char_styles.iter().filter(|c| c.chars.font_style.as_deref().is_some_and(|f| f.to_lowercase().contains("bold"))).map(|c| c.name.clone()).collect();
    let mut out = String::new();
    let runs: Vec<(std::ops::Range<usize>, (bool, bool))> = st
        .runs()
        .map(|(r, f)| {
            let fs = f.over.font_style.as_deref().unwrap_or("").to_lowercase();
            let it = fs.contains("italic") || fs.contains("oblique") || italico_estilo.contains(&f.style);
            let ng = fs.contains("bold") || negrito_estilo.contains(&f.style);
            (r, (it, ng))
        })
        .collect();
    for (pi, r) in st.para_ranges().into_iter().enumerate() {
        let estilo = st.paras.get(pi).map(|f| f.style.as_str()).unwrap_or("");
        let titulo = titulo_estilo(estilo);
        if titulo {
            out.push_str("# ");
            let t: String = st.text.get(r.clone()).unwrap_or("").chars().map(|c| if c == '\u{2028}' || c == '\t' { ' ' } else { c }).collect();
            out.push_str(t.trim());
            out.push('\n');
            continue;
        }
        for (rr, (it, ng)) in &runs {
            let a = rr.start.max(r.start);
            let b = rr.end.min(r.end);
            if a >= b {
                continue;
            }
            let t: String = st.text.get(a..b).unwrap_or("").chars().map(|c| if c == '\u{2028}' { ' ' } else { c }).collect();
            let marca = match (it, ng) {
                (true, true) => "***",
                (true, false) => "*",
                (false, true) => "**",
                _ => "",
            };
            // A marca fica colada no texto (os espaços das pontas ficam de fora).
            let ini = t.len() - t.trim_start().len();
            let fim = t.trim_end().len();
            if marca.is_empty() || t.trim().is_empty() {
                out.push_str(&t);
            } else {
                out.push_str(&t[..ini]);
                out.push_str(marca);
                out.push_str(&t[ini..fim]);
                out.push_str(marca);
                out.push_str(&t[fim..]);
            }
        }
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn novo_livro_e_diagramar_de_novo() {
        let mut s = Session::new();
        s.execute("livro.novo", &json!({"titulo": "Escama", "autor": "Nyck"})).unwrap();
        let l = s.execute("livro.obter", &json!({})).unwrap();
        assert_eq!(l["dados"]["titulo"], "Escama");
        let mut l: Livro = serde_json::from_value(l).unwrap();
        l.capitulos.push(designcraft_doc::livro::Capitulo { titulo: "Um".into(), texto: "Texto do capítulo.".into(), ..Default::default() });
        let r = s.execute("livro.diagramar", &json!({"livro": l})).unwrap();
        assert!(r["paginas"].as_u64().unwrap() >= 4);
        s.execute("edit.undo", &json!({})).unwrap();
    }

    #[test]
    fn texto_simples_vira_capitulos() {
        let t = texto_marcado("a.txt", "Capítulo 1\nUm *dois*.\nCapítulo 2\nTrês.".as_bytes()).unwrap();
        let c = Livro::capitulos_do_texto(&t);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].texto, "Um *dois*.");
    }

    #[test]
    fn arquivo_que_nao_existe_e_erro() {
        let mut s = Session::new();
        s.execute("livro.novo", &json!({})).unwrap();
        assert!(s.execute("livro.importar_texto", &json!({"path": "/nao/existe.docx"})).is_err());
    }
}
