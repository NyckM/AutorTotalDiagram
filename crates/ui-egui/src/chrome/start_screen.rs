use super::{Color32, Content, DesignApp, Sense, Stroke, Tokens, json, semibold, vec2};

pub fn start_screen(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let r = ui.available_rect_before_wrap();
    ui.painter().rect_filled(r, 0.0, t.panel_darker);
    ui.scope_builder(egui::UiBuilder::new().max_rect(r.shrink(48.0)).layout(egui::Layout::top_down(egui::Align::Min)), |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.label(egui::RichText::new("Diagrama do Autor Total").font(semibold(30.0)).color(t.text_strong));
            ui.label(egui::RichText::new("Diagramação de livros, do texto ao PDF da gráfica, com o Mentor do Autor Total ao lado.").size(15.0).color(t.text_dim));
            ui.add_space(22.0);
            ui.horizontal(|ui| {
                let grande = |texto: &str| egui::RichText::new(format!("  {texto}  ")).size(14.0);
                if ui
                    .add(egui::Button::new(grande("Novo livro").color(Color32::WHITE)).fill(t.accent_strong).corner_radius(16.0).min_size(vec2(0.0, 34.0)))
                    .clicked()
                {
                    let _ = app.run("livro.novo", json!({}));
                    let _ = app.run("window.panel", json!({"panel": "livro"}));
                        let _ = app.run("view.firstPage", json!({}));
                }
                if ui.add(egui::Button::new(grande("Abrir livro")).corner_radius(16.0).min_size(vec2(0.0, 34.0))).clicked() {
                    let _ = app.run("app.openDialog", json!({}));
                }
                if ui
                    .add(egui::Button::new(grande("Novo livro a partir de um texto (.docx, .txt)…")).corner_radius(16.0).min_size(vec2(0.0, 34.0)))
                    .clicked()
                {
                    novo_de_texto(app);
                }
            });
            ui.add_space(20.0);
            mentor_card(app, ui);
            recent_files(app, ui);
            ui.add_space(24.0);
            ui.label(egui::RichText::new("Começar um livro neste tamanho").font(semibold(15.0)).color(t.text_strong));
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                for tm in designcraft_doc::livro::TAMANHOS {
                    let (cr, resp) = ui.allocate_exact_size(vec2(132.0, 160.0), Sense::click());
                    let hov = resp.hovered();
                    ui.painter().rect_filled(cr, 6.0, if hov { t.hover } else { t.panel });
                    let k = (86.0 / tm.w.max(tm.h)) as f32;
                    let pr = egui::Rect::from_center_size(cr.center() - vec2(0.0, 18.0), vec2(tm.w as f32 * k, tm.h as f32 * k));
                    ui.painter().rect_filled(pr, 0.0, Color32::from_gray(245));
                    let (nome, uso) = tm.nome.split_once(" · ").unwrap_or((tm.nome, ""));
                    ui.painter().text(egui::pos2(cr.center().x, cr.max.y - 30.0), egui::Align2::CENTER_CENTER, nome, semibold(12.0), t.text_strong);
                    ui.painter().text(egui::pos2(cr.center().x, cr.max.y - 13.0), egui::Align2::CENTER_CENTER, uso, egui::FontId::proportional(10.5), t.text_dim);
                    if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        let _ = app.run("livro.novo", json!({"tamanho": tm.chave}));
                        let _ = app.run("window.panel", json!({"panel": "livro"}));
                        let _ = app.run("view.firstPage", json!({}));
                    }
                }
            });
        });
    });
    let _ = Content::Unassigned;
    let _ = Stroke::NONE;
}

/// Novo livro já com o texto: escolhe o arquivo, importa os capítulos e diagrama.
pub fn novo_de_texto(app: &mut DesignApp) {
    let r = app.ask_open("livroTexto", crate::app::file_dialogs::Bound::App, |app, path| {
        let titulo = std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        app.run("livro.novo", json!({"titulo": titulo}))?;
        app.run("livro.importar_texto", json!({"path": path}))?;
        let v = app.run("livro.diagramar", json!({}))?;
        let _ = app.run("window.panel", json!({"panel": "livro"}));
                        let _ = app.run("view.firstPage", json!({}));
        Ok(v)
    });
    if let Err(e) = r {
        app.status(e);
    }
}

/// O Mentor dá as boas-vindas (a primeira vez em destaque) e lembra que conversa pelo painel.
fn mentor_card(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let primeira = app.ui.mentor.boas_vindas_pendentes();
    egui::Frame::NONE.fill(t.panel).corner_radius(10.0).inner_margin(egui::Margin::same(16)).show(ui, |ui| {
        ui.set_max_width(760.0);
        crate::panels::mentor::boas_vindas(ui, &t);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(egui::RichText::new("  Conversar com o Mentor  ").color(Color32::WHITE)).fill(t.accent_strong).corner_radius(14.0))
                .clicked()
            {
                let _ = app.run("window.panel", json!({"panel": "mentor"}));
                app.ui.mentor.marcar_boas_vindas();
            }
            if primeira && ui.button("Entendi").clicked() {
                app.ui.mentor.marcar_boas_vindas();
            }
        });
    });
}

/// Recent files shown on the start screen (File › Open Recent has them all).
const START_SCREEN_RECENT: usize = 8;

/// Start screen: the most recent files as cards with the first page's picture (saved in the file)
/// and the name; the full path is the tooltip. A file that is gone is dimmed and says so when
/// clicked. Nothing when the list is empty.
fn recent_files(app: &mut DesignApp, ui: &mut egui::Ui) {
    if app.session.prefs.recent_files.is_empty() {
        return;
    }
    let t = Tokens::get(ui.ctx());
    ui.add_space(24.0);
    ui.label(crate::rtl::widget(
        ui,
        egui::RichText::new(crate::i18n::tr(&app.ui.language, "Recent files")).font(semibold(15.0)).color(t.text_strong),
    ));
    ui.add_space(8.0);
    let rows: Vec<(usize, String)> = app.session.prefs.recent_files.iter().take(START_SCREEN_RECENT).cloned().enumerate().collect();
    let mut open = None;
    ui.horizontal_wrapped(|ui| {
        for (i, path) in rows {
            let stat = recent_file_stat(ui.ctx(), &path);
            let there = stat.exists;
            let modified = there.then_some(stat.modified);
            let (cr, resp) = ui.allocate_exact_size(vec2(140.0, 170.0), Sense::click());
            let resp = resp.on_hover_text(&path);
            ui.painter().rect_filled(cr, 6.0, if resp.hovered() { t.hover } else { t.panel });
            let area = egui::Rect::from_min_size(cr.min + vec2(20.0, 12.0), vec2(100.0, 108.0));
            match modified.and_then(|m| recent_preview_texture(ui.ctx(), &path, m)) {
                Some(tex) => {
                    let size = tex.size_vec2();
                    let k = (area.width() / size.x).min(area.height() / size.y);
                    let pr = egui::Rect::from_center_size(area.center(), size * k);
                    ui.painter().image(tex.id(), pr, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                    ui.painter().rect_stroke(pr, 0.0, Stroke::new(1.0, t.border), egui::StrokeKind::Outside);
                }
                // No picture (saved before files had one) or the file is gone: a blank page.
                None => {
                    let pr = egui::Rect::from_center_size(area.center(), vec2(62.0, 80.0));
                    ui.painter().rect_filled(pr, 0.0, if there { Color32::from_gray(245) } else { Color32::from_gray(110) });
                }
            }
            let color = if there { t.text_strong } else { t.text_dim };
            // The name without `.designcraft`, on up to two lines (broken after a space or dash).
            let label = crate::menus::recent_label(&path);
            let name = label.strip_suffix(".designcraft").unwrap_or(&label).to_string();
            let mut job = egui::text::LayoutJob::simple(name, semibold(12.0), color, cr.width() - 12.0);
            job.wrap.max_rows = 2;
            job.halign = egui::Align::Center;
            let galley = ui.painter().layout_job(job);
            ui.painter().galley(egui::pos2(cr.center().x, area.max.y + 8.0), galley, color);
            if resp.clicked() {
                open = Some(i);
            }
        }
    });
    if let Some(i) = open {
        let _ = app.run("app.openRecent", json!({"index": i}));
    }
}

/// How often a recent file is looked up on disk again, in ms: the start screen and the Open Recent
/// menu draw every frame and must not touch the file system each time.
pub(super) const RECENT_STAT_MS: f64 = 3000.0;

/// A recent file as last seen on disk.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RecentStat {
    pub exists: bool,
    pub modified: Option<std::time::SystemTime>,
    checked: f64,
}

/// Whether recent file `path` is there and when it last changed, looked up again at most every
/// [`RECENT_STAT_MS`].
pub(crate) fn recent_file_stat(ctx: &egui::Context, path: &str) -> RecentStat {
    recent_file_stat_at(ctx, path, crate::now_ms())
}

pub(super) fn recent_file_stat_at(ctx: &egui::Context, path: &str, now: f64) -> RecentStat {
    let key = egui::Id::unique(("recent_stat", path));
    if let Some(st) = ctx.data(|d| d.get_temp::<RecentStat>(key))
        && (0.0..RECENT_STAT_MS).contains(&(now - st.checked))
    {
        return st;
    }
    let meta = designcraft_engine::io::metadata(path).ok().filter(designcraft_engine::io::Metadata::is_file);
    let st = RecentStat { exists: meta.is_some(), modified: meta.and_then(|m| m.modified().ok()), checked: now };
    ctx.data_mut(|d| d.insert_temp(key, st.clone()));
    st
}

/// The texture of a recent file's saved first-page picture, decoded once per file version.
fn recent_preview_texture(ctx: &egui::Context, path: &str, modified: Option<std::time::SystemTime>) -> Option<egui::TextureHandle> {
    type Cached = (Option<std::time::SystemTime>, Option<egui::TextureHandle>);
    let key = egui::Id::unique(("recent_preview", path));
    if let Some((m, tex)) = ctx.data(|d| d.get_temp::<Cached>(key))
        && m == modified
    {
        return tex;
    }
    let tex = designcraft_engine::cmd::recent_preview(path).and_then(|p| {
        let size = [usize::try_from(p.width).ok()?, usize::try_from(p.height).ok()?];
        (size[0].checked_mul(size[1])?.checked_mul(4)? == p.rgba.len()).then(|| {
            ctx.load_texture(format!("recent_preview:{path}"), egui::ColorImage::from_rgba_unmultiplied(size, &p.rgba), egui::TextureOptions::LINEAR)
        })
    });
    ctx.data_mut(|d| d.insert_temp::<Cached>(key, (modified, tex.clone())));
    tex
}

