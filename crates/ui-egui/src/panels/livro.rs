//! Painel Livro: dados, formato, partes e capítulos do livro, e o botão Diagramar. É o mesmo
//! conjunto do modo Diagramar da plataforma Autor Total.

use designcraft_doc::livro::{Capitulo, Livro, Lugar, PARTES, Parte, TAMANHOS};
use serde_json::json;

use crate::DesignApp;
use crate::theme::{Tokens, semibold};

#[derive(Clone, Debug, Default)]
pub struct Estado {
    /// Documento (uid) e revisão de onde o rascunho veio.
    doc: Option<u64>,
    rev: u64,
    livro: Livro,
    /// Há mudança ainda não guardada no documento.
    sujo: bool,
    capitulo: Option<usize>,
    pub aviso: String,
    /// Não refazer as páginas a cada mudança (para quem ajusta páginas à mão).
    manual: bool,
    /// Mudanças guardadas que ainda não estão nas páginas (no modo manual).
    pendente: bool,
}

const FONTES: &[&str] = &["Palatino Linotype", "EB Garamond", "Libre Baskerville", "Source Serif 4", "Book Antiqua", "Georgia", "Times New Roman", "Garamond"];
const CORPOS: &[(f64, &str)] = &[(0.0, "Automático"), (10.0, "10 pt"), (10.5, "10,5 pt"), (11.0, "11 pt"), (11.5, "11,5 pt"), (12.0, "12 pt")];
const ENTRELINHAS: &[(f64, &str)] = &[(1.3, "1,3"), (1.33, "1,33 (editorial)"), (1.4, "1,4"), (1.5, "1,5")];
const RECUOS: &[(f64, &str)] = &[(0.0, "Automático"), (0.5, "0,5 cm"), (0.8, "0,8 cm"), (1.2, "1,2 cm")];

/// Partes que têm texto para escrever (as outras são só ligar e desligar).
fn tem_texto(k: &str) -> bool {
    !matches!(k, "falsa" | "rosto" | "sumario")
}

fn dica(k: &str) -> &'static str {
    match k {
        "falsa" => "Página só com o título, antes da folha de rosto.",
        "rosto" => "Sempre presente: autor, título, subtítulo, edição, editora, cidade e ano.",
        "creditos" => "Verso da folha de rosto: direitos, edição, ISBN e equipe. Cole aqui a ficha catalográfica (CIP), se tiver: ela vai numa caixa.",
        "sumario" => "Gerado sozinho, com os números reais das páginas.",
        "epigrafe" => "A última linha começando com — vira a autoria da citação.",
        "notas" | "glossario" | "referencias" => "Cada linha é um item.",
        "sinopse" => "Vai na última página (quarta capa).",
        _ => "Cada quebra de linha é um parágrafo novo.",
    }
}

/// Recarrega o rascunho quando o documento ativo mudou (outro documento, desfazer…).
fn sincronizar(app: &mut DesignApp) {
    let Some(st) = app.session.active() else {
        app.ui.livro.doc = None;
        return;
    };
    let e = &mut app.ui.livro;
    if e.doc == Some(st.uid) && (e.sujo || e.rev == st.revision) {
        return;
    }
    e.doc = Some(st.uid);
    e.rev = st.revision;
    e.livro = st.doc.livro.as_deref().cloned().unwrap_or_else(|| Livro::novo(&st.doc.title, ""));
    e.sujo = false;
    if e.capitulo.is_some_and(|i| i >= e.livro.capitulos.len()) {
        e.capitulo = None;
    }
}

/// Guarda o rascunho no documento (entra no arquivo .diagramado e no desfazer).
fn guardar(app: &mut DesignApp) {
    if !app.ui.livro.sujo {
        return;
    }
    let l = app.ui.livro.livro.clone();
    match app.run("livro.definir", json!({"livro": l})) {
        Ok(_) => {
            app.ui.livro.sujo = false;
            if let Some(st) = app.session.active() {
                app.ui.livro.rev = st.revision;
            }
            // As páginas acompanham o painel: ligar a dedicatória já faz ela aparecer.
            if app.ui.livro.manual {
                app.ui.livro.pendente = true;
            } else {
                diagramar(app);
            }
        }
        Err(e) => app.ui.livro.aviso = e,
    }
}

pub fn diagramar(app: &mut DesignApp) {
    let l = app.ui.livro.livro.clone();
    match app.run("livro.diagramar", json!({"livro": l})) {
        Ok(v) => {
            app.ui.livro.sujo = false;
            app.ui.livro.pendente = false;
            app.ui.livro.aviso = format!("Livro diagramado: {} páginas.", v["paginas"].as_u64().unwrap_or(0));
            if let Some(st) = app.session.active() {
                app.ui.livro.rev = st.revision;
            }
        }
        Err(e) => app.ui.livro.aviso = e,
    }
}

pub fn importar(app: &mut DesignApp) {
    guardar(app);
    let r = app.ask_open("livroTexto", crate::app::file_dialogs::Bound::Document, |app, path| {
        let v = app.run("livro.importar_texto", json!({"path": path}))?;
        app.ui.livro.doc = None;
        let n = v["capitulos"].as_u64().unwrap_or(0);
        app.ui.livro.aviso = format!("{n} capítulo(s) importado(s). Clique em Diagramar o livro.");
        if !app.ui.livro.manual {
            let l = app.session.active().and_then(|st| st.doc.livro.as_deref().cloned());
            if let Some(l) = l {
                app.ui.livro.livro = l;
                diagramar(app);
                app.ui.livro.aviso = format!("{n} capítulo(s) importado(s) e diagramado(s).");
            }
        }
        Ok(v)
    });
    if let Err(e) = r {
        app.ui.livro.aviso = e;
    }
}

pub fn show(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    if app.session.active().is_none() {
        ui.label(egui::RichText::new("Nenhum livro aberto.").color(t.text_dim));
        if ui.button("Novo livro").clicked() {
            let _ = app.run("livro.novo", json!({}));
        }
        return;
    }
    sincronizar(app);
    let largura = ui.available_width() - 14.0;
    let botao = egui::Button::new(egui::RichText::new("Diagramar o livro").font(semibold(14.0)).color(egui::Color32::WHITE))
        .fill(t.accent_strong)
        .corner_radius(14.0)
        .min_size(egui::vec2(largura, 32.0));
    if ui.add(botao).on_hover_text("Monta todas as páginas a partir das partes e dos capítulos (Ctrl+Shift+D)").clicked() {
        diagramar(app);
    }
    let mut auto = !app.ui.livro.manual;
    if ui
        .checkbox(&mut auto, "Atualizar as páginas a cada mudança")
        .on_hover_text("Desligue antes de ajustar páginas à mão: diagramar de novo refaz todas as páginas e os ajustes manuais se perdem.")
        .changed()
    {
        app.ui.livro.manual = !auto;
    }
    if app.ui.livro.pendente {
        ui.label(egui::RichText::new("Há mudanças que ainda não estão nas páginas: clique em Diagramar o livro.").size(11.5).color(t.accent_strong));
    }
    if !app.ui.livro.aviso.is_empty() {
        ui.label(egui::RichText::new(app.ui.livro.aviso.clone()).size(11.5).color(t.text_dim));
    }
    ui.add_space(6.0);

    // O rascunho é editado aqui; mudanças em caixa de texto são guardadas ao sair do campo.
    let mut mudou = false;
    let mut saiu = false;
    let mut agora = false;
    let mut importar_pedido = false;
    {
        let e = &mut app.ui.livro;
        let l = &mut e.livro;
        let campo = |ui: &mut egui::Ui, rotulo: &str, v: &mut String, mudou: &mut bool, saiu: &mut bool| {
            ui.label(egui::RichText::new(rotulo).size(11.5).color(t.text_dim));
            let r = ui.add(egui::TextEdit::singleline(v).desired_width(largura));
            *mudou |= r.changed();
            *saiu |= r.lost_focus();
        };
        egui::CollapsingHeader::new(egui::RichText::new("Dados do livro").font(semibold(13.0))).default_open(true).show(ui, |ui| {
            let d = &mut l.dados;
            campo(ui, "Título", &mut d.titulo, &mut mudou, &mut saiu);
            campo(ui, "Subtítulo", &mut d.subtitulo, &mut mudou, &mut saiu);
            campo(ui, "Autor", &mut d.autor, &mut mudou, &mut saiu);
            campo(ui, "Editora", &mut d.editora, &mut mudou, &mut saiu);
            campo(ui, "Edição", &mut d.edicao, &mut mudou, &mut saiu);
            campo(ui, "Cidade", &mut d.cidade, &mut mudou, &mut saiu);
            campo(ui, "Ano", &mut d.ano, &mut mudou, &mut saiu);
            campo(ui, "ISBN", &mut d.isbn, &mut mudou, &mut saiu);
            campo(ui, "Organização", &mut d.organizacao, &mut mudou, &mut saiu);
            ui.label(egui::RichText::new("Equipe (vai na página de créditos, uma pessoa por linha)").size(11.5).color(t.text_dim));
            let r = ui.add(egui::TextEdit::multiline(&mut d.equipe).desired_rows(3).desired_width(largura));
            mudou |= r.changed();
            saiu |= r.lost_focus();
        });
        egui::CollapsingHeader::new(egui::RichText::new("Formato e tipografia").font(semibold(13.0))).default_open(true).show(ui, |ui| {
            let f = &mut l.formato;
            let antes = f.clone();
            ui.label(egui::RichText::new("Modelo de diagramação").size(11.5).color(t.text_dim));
            egui::ComboBox::from_id_salt("livro-modelo")
                .width(largura)
                .selected_text(if f.modelo == "classico" { "Clássico" } else { "Editorial (padrão profissional)" })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut f.modelo, "editorial".to_string(), "Editorial (padrão profissional)");
                    ui.selectable_value(&mut f.modelo, "classico".to_string(), "Clássico");
                });
            ui.label(egui::RichText::new("Tamanho da página").size(11.5).color(t.text_dim));
            let nome = designcraft_doc::livro::tamanho(&f.tamanho).nome;
            egui::ComboBox::from_id_salt("livro-tamanho").width(largura).selected_text(nome).show_ui(ui, |ui| {
                for tm in TAMANHOS {
                    ui.selectable_value(&mut f.tamanho, tm.chave.to_string(), tm.nome);
                }
            });
            ui.label(egui::RichText::new("Fonte").size(11.5).color(t.text_dim));
            egui::ComboBox::from_id_salt("livro-fonte").width(largura).selected_text(f.fonte.clone()).show_ui(ui, |ui| {
                for nome in FONTES {
                    ui.selectable_value(&mut f.fonte, nome.to_string(), *nome);
                }
            });
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Corpo").size(11.5).color(t.text_dim));
                    let atual = CORPOS.iter().find(|c| (c.0 - f.corpo).abs() < 0.01).map_or("Automático", |c| c.1);
                    egui::ComboBox::from_id_salt("livro-corpo").width(largura / 2.0 - 8.0).selected_text(atual).show_ui(ui, |ui| {
                        for (v, n) in CORPOS {
                            ui.selectable_value(&mut f.corpo, *v, *n);
                        }
                    });
                });
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Entrelinha").size(11.5).color(t.text_dim));
                    let atual = ENTRELINHAS.iter().find(|c| (c.0 - f.entrelinha).abs() < 0.001).map_or("1,33 (editorial)", |c| c.1);
                    egui::ComboBox::from_id_salt("livro-entrelinha").width(largura / 2.0 - 8.0).selected_text(atual).show_ui(ui, |ui| {
                        for (v, n) in ENTRELINHAS {
                            ui.selectable_value(&mut f.entrelinha, *v, *n);
                        }
                    });
                });
            });
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Margens").size(11.5).color(t.text_dim));
                    egui::ComboBox::from_id_salt("livro-margem")
                        .width(largura / 2.0 - 8.0)
                        .selected_text(if f.margem == "ampla" { "Amplas" } else { "Padrão" })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut f.margem, "normal".to_string(), "Padrão");
                            ui.selectable_value(&mut f.margem, "ampla".to_string(), "Amplas");
                        });
                });
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Recuo do parágrafo").size(11.5).color(t.text_dim));
                    let atual = RECUOS.iter().find(|c| (c.0 - f.recuo).abs() < 0.001).map_or("Automático", |c| c.1);
                    egui::ComboBox::from_id_salt("livro-recuo").width(largura / 2.0 - 8.0).selected_text(atual).show_ui(ui, |ui| {
                        for (v, n) in RECUOS {
                            ui.selectable_value(&mut f.recuo, *v, *n);
                        }
                    });
                });
            });
            ui.checkbox(&mut f.capitulos_na_direita, "Capítulos começam na página da direita");
            ui.checkbox(&mut f.numerar_capitulos, "Numerar capítulos (Capítulo 1, 2…)");
            ui.checkbox(&mut f.capitular, "Letra capitular na abertura");
            ui.checkbox(&mut f.capitular_partes, "Capitular também no Prefácio, Apresentação, Introdução, Conclusão e Posfácio");
            ui.checkbox(&mut f.sem_recuo_partes, "Primeiro parágrafo sem recuo no Prefácio, Introdução e demais partes em prosa");
            ui.checkbox(&mut f.cabecalho, "Cabeçalho (autor e título do capítulo)");
            ui.checkbox(&mut f.coletanea, "Coletânea de vários autores (o nome logo abaixo do título é o autor do texto)");
            if *f != antes {
                mudou = true;
                agora = true;
            }
        });
        let mut partes = |ui: &mut egui::Ui, lugar: Lugar, mudou: &mut bool, saiu: &mut bool, agora: &mut bool| {
            for (k, nome, lg, _) in PARTES {
                if *lg != lugar {
                    continue;
                }
                let p = l.partes.entry((*k).to_string()).or_insert_with(Parte::default);
                let mut ligada = p.ligada || *k == "rosto";
                let r = ui.add_enabled(*k != "rosto", egui::Checkbox::new(&mut ligada, *nome));
                if r.changed() {
                    p.ligada = ligada;
                    *mudou = true;
                    *agora = true;
                }
                if ligada {
                    ui.label(egui::RichText::new(dica(k)).size(11.0).color(t.text_dim));
                    if tem_texto(k) {
                        let r = ui.add(egui::TextEdit::multiline(&mut p.texto).desired_rows(3).desired_width(largura));
                        *mudou |= r.changed();
                        *saiu |= r.lost_focus();
                    }
                    ui.add_space(4.0);
                }
            }
        };
        egui::CollapsingHeader::new(egui::RichText::new("Antes do texto").font(semibold(13.0))).default_open(false).show(ui, |ui| {
            partes(ui, Lugar::Antes, &mut mudou, &mut saiu, &mut agora);
        });
        egui::CollapsingHeader::new(egui::RichText::new(format!("Capítulos ({})", l.capitulos.len())).font(semibold(13.0))).default_open(true).show(
            ui,
            |ui| {
                if ui.button("Importar texto (.docx, .txt, .md, .rtf)…").on_hover_text("Cada Título 1 do Word (ou linha \"Capítulo 1\") vira um capítulo.").clicked() {
                    importar_pedido = true;
                }
                let l_coletanea = l.formato.coletanea;
                let mut mover: Option<(usize, bool)> = None;
                let mut apagar: Option<usize> = None;
                for (i, c) in l.capitulos.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let aberto = e.capitulo == Some(i);
                        if ui.selectable_label(aberto, if aberto { "▾" } else { "▸" }).clicked() {
                            e.capitulo = if aberto { None } else { Some(i) };
                        }
                        let r = ui.add(egui::TextEdit::singleline(&mut c.titulo).hint_text("(sem título)").desired_width(largura - 96.0));
                        mudou |= r.changed();
                        saiu |= r.lost_focus();
                        if ui.small_button("↑").on_hover_text("Subir").clicked() {
                            mover = Some((i, true));
                        }
                        if ui.small_button("↓").on_hover_text("Descer").clicked() {
                            mover = Some((i, false));
                        }
                        if ui.small_button("×").on_hover_text("Apagar o capítulo").clicked() {
                            apagar = Some(i);
                        }
                    });
                    if e.capitulo == Some(i) {
                        if l_coletanea {
                            ui.label(egui::RichText::new("Autor do texto").size(11.0).color(t.text_dim));
                            let r = ui.add(egui::TextEdit::singleline(&mut c.autor).desired_width(largura));
                            mudou |= r.changed();
                            saiu |= r.lost_focus();
                        }
                        let r = ui.add(egui::TextEdit::multiline(&mut c.texto).desired_rows(10).desired_width(largura));
                        mudou |= r.changed();
                        saiu |= r.lost_focus();
                        ui.label(egui::RichText::new("*itálico*  **negrito**  · cada linha é um parágrafo · * * * separa cenas").size(10.5).color(t.text_dim));
                    }
                }
                if let Some((i, sobe)) = mover {
                    let j = if sobe { i.checked_sub(1) } else { Some(i + 1).filter(|j| *j < l.capitulos.len()) };
                    if let Some(j) = j {
                        l.capitulos.swap(i, j);
                        mudou = true;
                        agora = true;
                    }
                }
                if let Some(i) = apagar
                    && i < l.capitulos.len()
                {
                    l.capitulos.remove(i);
                    e.capitulo = None;
                    mudou = true;
                    agora = true;
                }
                if ui.button("+ Adicionar capítulo").clicked() {
                    l.capitulos.push(Capitulo { titulo: format!("Capítulo {}", l.capitulos.len() + 1), ..Default::default() });
                    e.capitulo = Some(l.capitulos.len() - 1);
                    mudou = true;
                    agora = true;
                }
            },
        );
        egui::CollapsingHeader::new(egui::RichText::new("Depois do texto").font(semibold(13.0))).default_open(false).show(ui, |ui| {
            partes(ui, Lugar::Depois, &mut mudou, &mut saiu, &mut agora);
        });
        if mudou {
            e.sujo = true;
        }
    }
    if importar_pedido {
        importar(app);
    }
    if (saiu || agora) && app.ui.livro.sujo {
        guardar(app);
    }
}
