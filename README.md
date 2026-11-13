# Diagrama do Autor Total

Programa de computador para **diagramar livros**, do texto ao PDF da gráfica ou da Amazon KDP, com o
**Mentor do Autor Total a bordo**. Em português. Os arquivos são salvos como **`.diagramado`**.

Feito a partir do [DesignCraft](https://github.com/storytold/designcraft) (MIT OR Apache-2.0), um
programa de editoração em Rust. O Diagrama mantém o motor de composição dele (parágrafos, hifenização,
PDF com fontes embutidas) e acrescenta o que é do Autor Total: o painel **Livro**, o **Mentor** e o
formato `.diagramado`. Veja [README-DesignCraft.md](README-DesignCraft.md), `NOTICE` e as licenças.

## O que faz

- **Novo livro** (Ctrl+N), pelos tamanhos de livro: 14 × 21, 15,5 × 23, 16 × 23, A5, bolso 11 × 18, A4
  e os da Amazon KDP (5 × 8", 5,5 × 8,5", 6 × 9").
- **Importar o texto** (.docx, .txt, .md, .rtf): cada *Título 1* do Word (ou linha "Capítulo 1",
  "CAPÍTULO II – Nome", ou `# Título`) vira um capítulo. Itálico e negrito vêm junto. Um título
  quebrado em duas linhas (dois Título 1 seguidos) vira um título só.
- **Painel Livro**, igual ao modo Diagramar da plataforma:
  - Dados: título, subtítulo, autor, editora, edição, cidade, ano, ISBN, organização e equipe;
  - Formato: modelo Editorial ou Clássico, tamanho, fonte, corpo, entrelinha, margens, recuo,
    capítulos na página da direita, numerar capítulos, letra capitular, capitular também no Prefácio
    e na Introdução, primeiro parágrafo sem recuo nas partes em prosa, cabeçalho;
  - Antes do texto: falsa folha de rosto, folha de rosto, créditos e ficha catalográfica, dedicatória,
    epígrafe, **sumário com as páginas reais**, prefácio, apresentação, introdução;
  - Capítulos: importar, renomear, reordenar, editar o texto, acrescentar e apagar;
  - Depois do texto: conclusão, posfácio, agradecimentos, notas, glossário, referências, sobre o autor
    e **sinopse (quarta capa)**.
- **Diagramar o livro** (Ctrl+Shift+D): monta todas as páginas: páginas-mestre com cabeçalho (autor
  nas pares, título do capítulo nas ímpares) e número da página, abertura de capítulo sem cabeçalho,
  capitular (que diminui sozinha quando o primeiro parágrafo é curto), separadores de cena
  (`* * *`), hifenização em português, páginas em branco para cada capítulo abrir na direita.
- Depois de diagramado, **tudo é editável à mão** (texto, estilos, páginas, imagens) com as
  ferramentas do DesignCraft, e exporta **PDF** (com sangria e marcas de corte, PDF/X), EPUB, PNG e IDML.
- **Mentor a bordo** (painel Mentor e botão Mentor no topo): a mesma conversa do Autor Total Escrita.
  Diagramar funciona sem internet; o Mentor precisa de internet e da conta cadastrada em
  autortotal.com.br (login com e-mail e senha, com verificação em duas etapas quando a conta tem).
  O Mentor recebe o texto do livro aberto para responder sobre ele.


