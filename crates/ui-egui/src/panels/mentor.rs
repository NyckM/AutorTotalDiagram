//! Mentor a bordo: a conversa com o Mentor do Autor Total, igual à do Autor Total Escrita.
//! Diagramar funciona sem internet; o Mentor precisa de internet e da conta cadastrada na
//! plataforma (autortotal.com.br). O login e a trava de acesso são conferidos no servidor.

use std::sync::mpsc::{Receiver, channel};

use serde_json::{Value, json};

use crate::DesignApp;
use crate::theme::{Tokens, semibold};

const SB_URL_PADRAO: &str = "https://wbwlhxwfhrgaikettdev.supabase.co";

/// O servidor do Autor Total (`DIAGRAMA_SB_URL` troca, para testes).
fn sb_url() -> String {
    #[cfg(test)]
    if let Some(u) = URL_TESTE.get() {
        return u.clone();
    }
    std::env::var("DIAGRAMA_SB_URL").ok().filter(|u| u.starts_with("http")).unwrap_or_else(|| SB_URL_PADRAO.to_string())
}
const SB_KEY: &str = "sb_publishable_Rgg8mN05JXMumeE5amE5hA_G7PI94hr";
#[cfg(test)]
static URL_TESTE: std::sync::OnceLock<String> = std::sync::OnceLock::new();

const PERFIL: &str = "A pessoa está usando o Diagrama do Autor Total, o programa de computador do Autor Total para diagramar livros \
(tamanho da página, margens, fonte, entrelinha, capitular, sumário, folha de rosto, créditos, dedicatória, epígrafe, prefácio, \
introdução, capítulos e quarta capa). Ajude com dúvidas de diagramação, de preparação do livro para gráfica ou para a Amazon KDP, \
e com o texto do livro, que vai junto.";
/// Mensagens anteriores que vão junto (como no app).
const HISTORICO: usize = 8;
const MAX_LIVRO: usize = 200_000;

#[derive(Clone, Default)]
struct Sessao {
    email: String,
    access: String,
    refresh: String,
    /// Segundos (relógio do sistema) em que o token vence.
    vence: u64,
}

enum Resposta {
    Login(Result<(Sessao, Option<String>), String>),
    Codigo(Result<Sessao, String>),
    Mentor(Result<String, String>, Option<Sessao>),
}

#[derive(Default)]
pub struct Estado {
    carregado: bool,
    sessao: Option<Sessao>,
    email: String,
    senha: String,
    /// Verificação em duas etapas: o fator a confirmar e o código digitado.
    fator: Option<String>,
    codigo: String,
    entrada: String,
    /// (é do autor, texto)
    conversa: Vec<(bool, String)>,
    ocupado: bool,
    erro: String,
    rx: Option<Receiver<Resposta>>,
    /// Boas-vindas (onboarding) já vistas.
    pub boas_vindas_vistas: bool,
}

impl Clone for Estado {
    /// A conversa em andamento (e o canal da resposta) fica com o original.
    fn clone(&self) -> Self {
        Estado { carregado: false, boas_vindas_vistas: self.boas_vindas_vistas, ..Default::default() }
    }
}

impl std::fmt::Debug for Estado {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mentor").field("conectado", &self.sessao.is_some()).field("mensagens", &self.conversa.len()).finish()
    }
}

fn agora() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Pasta das preferências do Diagrama (onde fica a sessão do Mentor).
fn pasta() -> Option<std::path::PathBuf> {
    let env = |k: &str| std::env::var_os(k).map(std::path::PathBuf::from);
    if cfg!(target_os = "windows") {
        env("APPDATA").map(|p| p.join("DiagramaAutorTotal"))
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|p| p.join("Library/Application Support/DiagramaAutorTotal"))
    } else {
        env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|h| h.join(".config"))).map(|p| p.join("diagrama-autor-total"))
    }
}

fn arquivo() -> Option<std::path::PathBuf> {
    pasta().map(|p| p.join("mentor.json"))
}

impl Estado {
    fn carregar(&mut self) {
        if std::mem::replace(&mut self.carregado, true) {
            return;
        }
        let Some(f) = arquivo() else { return };
        let Ok(t) = std::fs::read_to_string(f) else { return };
        let Ok(v) = serde_json::from_str::<Value>(&t) else { return };
        self.boas_vindas_vistas = v["boasVindas"].as_bool().unwrap_or(false);
        if let (Some(e), Some(r)) = (v["email"].as_str(), v["refresh"].as_str())
            && !r.is_empty()
        {
            self.email = e.to_string();
            self.sessao = Some(Sessao { email: e.into(), access: String::new(), refresh: r.into(), vence: 0 });
        }
    }

    fn guardar(&self) {
        let Some(p) = pasta() else { return };
        let _ = std::fs::create_dir_all(&p);
        let v = json!({
            "email": self.sessao.as_ref().map(|s| s.email.as_str()).unwrap_or(""),
            "refresh": self.sessao.as_ref().map(|s| s.refresh.as_str()).unwrap_or(""),
            "boasVindas": self.boas_vindas_vistas,
        });
        let _ = std::fs::write(p.join("mentor.json"), v.to_string());
    }

    pub fn marcar_boas_vindas(&mut self) {
        self.carregar();
        self.boas_vindas_vistas = true;
        self.guardar();
    }

    pub fn boas_vindas_pendentes(&mut self) -> bool {
        self.carregar();
        !self.boas_vindas_vistas
    }
}

fn agente() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(std::time::Duration::from_secs(180)))
        .build()
        .new_agent()
}

/// POST com JSON; devolve (status, corpo em JSON).
fn post(url: &str, token: Option<&str>, corpo: &Value) -> Result<(u16, Value), String> {
    let mut r = agente().post(url).header("apikey", SB_KEY).header("Content-Type", "application/json");
    if let Some(t) = token {
        r = r.header("Authorization", &format!("Bearer {t}"));
    }
    let mut resp = r.send(corpo.to_string()).map_err(|e| format!("sem conexão com o Autor Total ({e})"))?;
    let status = resp.status().as_u16();
    let texto = resp.body_mut().with_config().limit(8 * 1024 * 1024).read_to_string().map_err(|e| e.to_string())?;
    Ok((status, serde_json::from_str(&texto).unwrap_or(Value::Null)))
}

fn mensagem_de_erro(v: &Value, padrao: &str) -> String {
    for k in ["error_description", "msg", "error", "message"] {
        if let Some(s) = v[k].as_str().filter(|s| !s.is_empty()) {
            return traduzir(s);
        }
    }
    padrao.to_string()
}

fn traduzir(s: &str) -> String {
    match s {
        "Invalid login credentials" => "E-mail ou senha incorretos.".into(),
        "Email not confirmed" => "Confirme seu e-mail pelo link que o Autor Total enviou.".into(),
        _ => s.into(),
    }
}

fn sessao_de(v: &Value, email: &str) -> Option<Sessao> {
    let access = v["access_token"].as_str()?.to_string();
    let refresh = v["refresh_token"].as_str().unwrap_or("").to_string();
    let vence = agora() + v["expires_in"].as_u64().unwrap_or(3600).saturating_sub(60);
    let email = v["user"]["email"].as_str().unwrap_or(email).to_string();
    Some(Sessao { email, access, refresh, vence })
}

/// Fator de verificação em duas etapas já confirmado (o código vai ser pedido).
fn fator_verificado(v: &Value) -> Option<String> {
    v["user"]["factors"].as_array()?.iter().find(|f| f["status"] == "verified").and_then(|f| f["id"].as_str()).map(str::to_string)
}

fn login(email: &str, senha: &str) -> Result<(Sessao, Option<String>), String> {
    let (st, v) = post(&format!("{}/auth/v1/token?grant_type=password", sb_url()), None, &json!({"email": email, "password": senha}))?;
    if st >= 400 {
        return Err(mensagem_de_erro(&v, "Não consegui entrar."));
    }
    let s = sessao_de(&v, email).ok_or("Resposta inesperada do servidor.")?;
    Ok((s, fator_verificado(&v)))
}

fn renovar(s: &Sessao) -> Result<Sessao, String> {
    let (st, v) = post(&format!("{}/auth/v1/token?grant_type=refresh_token", sb_url()), None, &json!({"refresh_token": s.refresh}))?;
    if st >= 400 {
        return Err("Sua sessão expirou. Entre de novo.".into());
    }
    sessao_de(&v, &s.email).ok_or_else(|| "Resposta inesperada do servidor.".into())
}

fn confirmar_codigo(s: &Sessao, fator: &str, codigo: &str) -> Result<Sessao, String> {
    let (st, v) = post(&format!("{}/auth/v1/factors/{fator}/challenge", sb_url()), Some(&s.access), &json!({}))?;
    if st >= 400 {
        return Err(mensagem_de_erro(&v, "Não consegui pedir a verificação."));
    }
    let desafio = v["id"].as_str().ok_or("Resposta inesperada do servidor.")?;
    let (st, v) = post(&format!("{}/auth/v1/factors/{fator}/verify", sb_url()), Some(&s.access), &json!({"challenge_id": desafio, "code": codigo.trim()}))?;
    if st >= 400 {
        return Err(mensagem_de_erro(&v, "Código incorreto."));
    }
    sessao_de(&v, &s.email).ok_or_else(|| "Resposta inesperada do servidor.".into())
}

fn perguntar(s: &Sessao, pergunta: &str, historico: Vec<Value>, livro: String) -> (Result<String, String>, Option<Sessao>) {
    let mut nova = None;
    let mut s = s.clone();
    if s.access.is_empty() || agora() >= s.vence {
        match renovar(&s) {
            Ok(n) => {
                s = n.clone();
                nova = Some(n);
            }
            Err(e) => return (Err(e), None),
        }
    }
    let corpo = json!({
        "question": pergunta,
        "history": historico,
        "project_id": Value::Null,
        "historico": "mentor",
        "conto": livro,
        "perfil": PERFIL,
        "origem": "desktop",
        "stream": false,
    });
    let r = match post(&format!("{}/functions/v1/scriba", sb_url()), Some(&s.access), &corpo) {
        Err(e) => Err(e),
        Ok((st, v)) if st >= 400 || v["answer"].as_str().is_none() => Err(mensagem_de_erro(&v, &format!("O Mentor não respondeu (erro {st})."))),
        Ok((_, v)) => Ok(v["answer"].as_str().unwrap_or("").to_string()),
    };
    (r, nova)
}

/// O texto do livro aberto, para o Mentor conhecer o livro.
fn texto_do_livro(app: &DesignApp) -> String {
    let Some(st) = app.session.active() else { return String::new() };
    let mut t = String::new();
    if let Some(l) = st.doc.livro.as_deref() {
        t.push_str(&format!("Título: {}\nAutor: {}\n", l.dados.titulo, l.dados.autor));
        for (k, p) in &l.partes {
            if p.ligada && !p.texto.trim().is_empty() {
                t.push_str(&format!("\n[{k}]\n{}\n", p.texto));
            }
        }
        for c in &l.capitulos {
            t.push_str(&format!("\n# {}\n{}\n", c.titulo, c.texto));
            if t.len() > MAX_LIVRO {
                break;
            }
        }
    }
    let mut i = MAX_LIVRO.min(t.len());
    while i > 0 && !t.is_char_boundary(i) {
        i -= 1;
    }
    t.truncate(i);
    t
}

fn enviar(app: &mut DesignApp, ctx: &egui::Context) {
    let pergunta = app.ui.mentor.entrada.trim().to_string();
    if pergunta.is_empty() || app.ui.mentor.ocupado {
        return;
    }
    let Some(s) = app.ui.mentor.sessao.clone() else { return };
    let historico: Vec<Value> = app
        .ui
        .mentor
        .conversa
        .iter()
        .rev()
        .take(HISTORICO)
        .rev()
        .map(|(autor, t)| json!({"role": if *autor { "user" } else { "model" }, "content": t}))
        .collect();
    let livro = texto_do_livro(app);
    let m = &mut app.ui.mentor;
    m.conversa.push((true, pergunta.clone()));
    m.entrada.clear();
    m.ocupado = true;
    m.erro.clear();
    let (tx, rx) = channel();
    m.rx = Some(rx);
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        let (r, nova) = perguntar(&s, &pergunta, historico, livro);
        let _ = tx.send(Resposta::Mentor(r, nova));
        ctx.request_repaint();
    });
}

fn receber(m: &mut Estado) {
    let Some(rx) = &m.rx else { return };
    let Ok(r) = rx.try_recv() else { return };
    m.rx = None;
    m.ocupado = false;
    match r {
        Resposta::Login(Ok((s, fator))) => {
            m.senha.clear();
            m.sessao = Some(s);
            m.fator = fator;
            if m.fator.is_none() {
                m.guardar();
            }
        }
        Resposta::Login(Err(e)) | Resposta::Codigo(Err(e)) => m.erro = e,
        Resposta::Codigo(Ok(s)) => {
            m.sessao = Some(s);
            m.fator = None;
            m.codigo.clear();
            m.guardar();
        }
        Resposta::Mentor(r, nova) => {
            if let Some(n) = nova {
                m.sessao = Some(n);
                m.guardar();
            }
            match r {
                Ok(t) => m.conversa.push((false, t)),
                Err(e) => {
                    if e.contains("expirou") {
                        m.sessao = None;
                        m.guardar();
                    }
                    m.erro = e;
                }
            }
        }
    }
}

/// Boas-vindas do Mentor: o que o Diagrama faz, em 4 passos.
pub fn boas_vindas(ui: &mut egui::Ui, t: &Tokens) {
    ui.label(egui::RichText::new("Olá! Eu sou o Mentor do Autor Total.").font(semibold(15.0)).color(t.text_strong));
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Vou te acompanhar na diagramação do seu livro. É assim que funciona:").color(t.text_dim),
    );
    ui.add_space(6.0);
    for (n, passo) in [
        "Crie um livro novo e escolha o tamanho da página (14 × 21, 16 × 23, KDP…).",
        "No painel Livro, importe o texto (.docx ou .txt): cada Título 1 vira um capítulo.",
        "Ligue as partes que quiser: dedicatória, epígrafe, prefácio, introdução, sumário, sobre o autor, sinopse.",
        "Clique em Diagramar o livro e exporte o PDF para a gráfica ou para a Amazon KDP.",
    ]
    .iter()
    .enumerate()
    {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(format!("{}.", n + 1)).font(semibold(13.0)).color(t.accent_strong));
            ui.label(egui::RichText::new(*passo).color(t.text_strong));
        });
    }
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Diagramar funciona sem internet. Para conversar comigo, entre com a sua conta do Autor Total.")
            .size(11.5)
            .color(t.text_dim),
    );
}

pub fn show(app: &mut DesignApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    app.ui.mentor.carregar();
    receber(&mut app.ui.mentor);
    if app.ui.mentor.ocupado {
        ctx.request_repaint_after(std::time::Duration::from_millis(300));
    }
    let largura = ui.available_width();
    if app.ui.mentor.sessao.is_none() || app.ui.mentor.fator.is_some() {
        if app.ui.mentor.fator.is_none() {
            boas_vindas(ui, &t);
            ui.add_space(10.0);
            let m = &mut app.ui.mentor;
            ui.label("E-mail");
            ui.add(egui::TextEdit::singleline(&mut m.email).desired_width(largura));
            ui.label("Senha");
            let r = ui.add(egui::TextEdit::singleline(&mut m.senha).password(true).desired_width(largura));
            let entrar = ui.add_enabled(!m.ocupado, egui::Button::new(if m.ocupado { "Entrando…" } else { "Entrar" })).clicked()
                || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            if entrar && !m.email.trim().is_empty() && !m.senha.is_empty() && !m.ocupado {
                let (email, senha) = (m.email.trim().to_string(), m.senha.clone());
                m.ocupado = true;
                m.erro.clear();
                let (tx, rx) = channel();
                m.rx = Some(rx);
                let c = ctx.clone();
                std::thread::spawn(move || {
                    let _ = tx.send(Resposta::Login(login(&email, &senha)));
                    c.request_repaint();
                });
            }
            ui.hyperlink_to("Ainda não tem conta? Cadastre-se no Autor Total", "https://autortotal.com.br/cadastro");
        } else {
            let m = &mut app.ui.mentor;
            ui.label(egui::RichText::new("Verificação em duas etapas").font(semibold(14.0)).color(t.text_strong));
            ui.label("Digite o código de 6 dígitos do seu aplicativo autenticador.");
            ui.add(egui::TextEdit::singleline(&mut m.codigo).desired_width(120.0));
            if ui.add_enabled(!m.ocupado, egui::Button::new("Confirmar")).clicked()
                && let (Some(s), Some(f)) = (m.sessao.clone(), m.fator.clone())
            {
                let codigo = m.codigo.clone();
                m.ocupado = true;
                m.erro.clear();
                let (tx, rx) = channel();
                m.rx = Some(rx);
                let c = ctx.clone();
                std::thread::spawn(move || {
                    let _ = tx.send(Resposta::Codigo(confirmar_codigo(&s, &f, &codigo)));
                    c.request_repaint();
                });
            }
            if ui.button("Cancelar").clicked() {
                m.sessao = None;
                m.fator = None;
            }
        }
        if !app.ui.mentor.erro.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(0xe0, 0x7a, 0x6a), app.ui.mentor.erro.clone());
        }
        return;
    }

    // Conectado: a conversa.
    let email = app.ui.mentor.sessao.as_ref().map(|s| s.email.clone()).unwrap_or_default();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("Conectado como {email}")).size(11.0).color(t.text_dim));
        if ui.small_button("Sair").clicked() {
            app.ui.mentor.sessao = None;
            app.ui.mentor.conversa.clear();
            app.ui.mentor.guardar();
        }
    });
    if app.ui.mentor.conversa.is_empty() {
        ui.add_space(6.0);
        ui.label(egui::RichText::new("Pergunte sobre a diagramação ou sobre o texto do seu livro.").color(t.text_dim));
        for s in ["Qual o melhor tamanho para um romance?", "Como preparar o PDF para a Amazon KDP?", "Minhas margens estão boas para gráfica?"] {
            if ui.small_button(s).clicked() {
                app.ui.mentor.entrada = s.to_string();
                enviar(app, &ctx);
            }
        }
    }
    let altura = (ui.available_height() - 120.0).max(160.0);
    egui::ScrollArea::vertical().max_height(altura).stick_to_bottom(true).auto_shrink([false, true]).show(ui, |ui| {
        for (autor, texto) in &app.ui.mentor.conversa {
            let (fundo, cor) = if *autor { (t.hover, t.text_strong) } else { (t.panel_darker, t.text_strong) };
            egui::Frame::NONE.fill(fundo).corner_radius(8.0).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                if !*autor {
                    ui.label(egui::RichText::new("Mentor").font(semibold(11.0)).color(t.accent_strong));
                }
                ui.label(egui::RichText::new(texto.replace("**", "")).color(cor));
            });
            ui.add_space(6.0);
        }
        if app.ui.mentor.ocupado {
            ui.label(egui::RichText::new("O Mentor está escrevendo…").italics().color(t.text_dim));
        }
    });
    if !app.ui.mentor.erro.is_empty() {
        ui.colored_label(egui::Color32::from_rgb(0xe0, 0x7a, 0x6a), app.ui.mentor.erro.clone());
    }
    let r = ui.add(egui::TextEdit::multiline(&mut app.ui.mentor.entrada).hint_text("Escreva sua pergunta…").desired_rows(3).desired_width(largura));
    let enter = r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
    if enter {
        // O Enter entrou no texto: tira antes de enviar.
        let e = app.ui.mentor.entrada.trim_end_matches('\n').to_string();
        app.ui.mentor.entrada = e;
    }
    let ocupado = app.ui.mentor.ocupado;
    if ui.add_enabled(!ocupado, egui::Button::new("Enviar ao Mentor")).clicked() || enter {
        enviar(app, &ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};

    /// Um servidor de mentira que responde como o Autor Total e devolve os pedidos recebidos.
    fn servidor(respostas: Vec<(u16, String)>) -> (String, std::sync::mpsc::Receiver<(String, String)>) {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", l.local_addr().unwrap());
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            for (st, corpo) in respostas {
                let (mut c, _) = l.accept().unwrap();
                let mut r = BufReader::new(c.try_clone().unwrap());
                let mut linha = String::new();
                r.read_line(&mut linha).unwrap();
                let mut tam = 0;
                let mut cab = String::new();
                loop {
                    let mut h = String::new();
                    r.read_line(&mut h).unwrap();
                    if h.trim().is_empty() {
                        break;
                    }
                    if let Some(v) = h.to_lowercase().strip_prefix("content-length:") {
                        tam = v.trim().parse().unwrap();
                    }
                    cab.push_str(&h);
                }
                let mut b = vec![0; tam];
                r.read_exact(&mut b).unwrap();
                tx.send((format!("{linha}{cab}"), String::from_utf8(b).unwrap())).unwrap();
                write!(c, "HTTP/1.1 {st} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{corpo}", corpo.len()).unwrap();
            }
        });
        (url, rx)
    }

    #[test]
    fn login_renovacao_e_pergunta_ao_mentor() {
        let (url, rx) = servidor(vec![
            (400, r#"{"error":"invalid_grant","error_description":"Invalid login credentials"}"#.into()),
            (200, r#"{"access_token":"A1","refresh_token":"R1","expires_in":3600,"user":{"email":"a@b.c","factors":[]}}"#.into()),
            (200, r#"{"access_token":"A2","refresh_token":"R2","expires_in":3600,"user":{"email":"a@b.c"}}"#.into()),
            (200, r#"{"answer":"Use 14 x 21."}"#.into()),
        ]);
        URL_TESTE.set(url).unwrap();
        assert_eq!(login("a@b.c", "x").err().unwrap(), "E-mail ou senha incorretos.");
        let (s, fator) = login("a@b.c", "certa").unwrap();
        assert!(fator.is_none());
        assert_eq!(s.access, "A1");
        let vencida = Sessao { vence: 0, ..s };
        let (r, nova) = perguntar(&vencida, "Qual tamanho?", vec![], "texto do livro".into());
        assert_eq!(r.unwrap(), "Use 14 x 21.");
        assert_eq!(nova.unwrap().access, "A2");
        let pedidos: Vec<(String, String)> = rx.try_iter().collect();
        assert!(pedidos[0].0.contains("grant_type=password"));
        assert!(pedidos[2].0.contains("grant_type=refresh_token"));
        assert!(pedidos[3].0.starts_with("POST /functions/v1/scriba"));
        assert!(pedidos[3].0.to_lowercase().contains("authorization: bearer a2"));
        let corpo: Value = serde_json::from_str(&pedidos[3].1).unwrap();
        assert_eq!(corpo["question"], "Qual tamanho?");
        assert_eq!(corpo["origem"], "desktop");
        assert_eq!(corpo["conto"], "texto do livro");
    }
}
