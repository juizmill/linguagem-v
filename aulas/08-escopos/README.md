# Aula 08 — Escopos

## Objetivo

Preparar o `environment` da Aula 06/07 para suportar **múltiplos escopos
aninhados** — infraestrutura necessária para blocos e funções (Aulas 11/13),
mesmo que hoje nenhuma sintaxe da Aurora ainda crie um escopo filho de
verdade. Trocar o único `HashMap<String, Value>` global por uma cadeia de
escopos (`Environment` com referência ao escopo "de fora").

## Conceitos

- **Modelo de cadeia pai-filho (`Option<Box<Environment>>`) vs. pilha
  (`Vec<HashMap>`).** Decisão de design tomada pelo aluno antes de qualquer
  código: um `Environment` guarda uma referência opcional ao escopo em que
  foi *criado* (`pai`), não ao escopo de onde foi *chamado*. É o que
  diferencia **escopo léxico** (baseado em onde o código foi escrito) de
  **escopo dinâmico** (baseado em quem chamou quem) — vai importar de
  verdade na Aula 13 (Funções), quando uma função precisa enxergar as
  variáveis de onde foi *definida*, não de onde foi *chamada*.
- **Struct auto-referenciada precisa de `Box`.** `Environment` não pode
  conter outro `Environment` "cru" dentro de si mesmo (o compilador não
  consegue calcular um tamanho finito para isso). `Box<Environment>` resolve
  porque é só um ponteiro de tamanho fixo para dados no heap — mesma ideia
  já usada em `Expr::Binary { left: Box<Expr>, ... }` na Aula 05.
- **Recursão só-leitura vs. recursão mutável.** `get` (`&self`) percorre a
  cadeia com `match &self.pai { Some(pai) => pai.get(name), ... }` — só
  precisa *olhar*. `assign` (`&mut self`) precisa **escrever** no nível da
  cadeia onde a variável foi declarada, que pode não ser o escopo atual;
  para recursão mutável através de `Option<Box<Environment>>`, o
  equivalente é `self.pai.as_deref_mut()`, que devolve
  `Option<&mut Environment>` em vez de `Option<&Environment>`.
- **`define` vs. `assign`, agora em cadeia.** Mesma distinção declaração/
  reatribuição da Aula 07 (`let` vs. `=`), só que `assign` agora precisa
  subir a cadeia de escopos procurando onde a variável existe antes de
  decidir se é erro; `define` sempre grava no escopo mais interno, sem
  checar nada.
- **Campo privado força uso da API pública.** `valores: HashMap<...>` (sem
  `pub`) significa que nada fora do `impl Environment` — nem `eval_stmt`,
  nem os testes, nem `main.rs` — consegue chamar `.insert()`/`.contains_key()`
  direto nele; só os métodos públicos (`get`/`define`/`assign`). O
  compilador reclama "field is private" como lembrete automático da regra.
- **Posse (`String`) vs. empréstimo (`&String`) revisitado.** `Stmt::Let {
  name, value }` desestruturado a partir de `&Stmt` entrega `name` como
  `&String` (emprestado da árvore do programa); `Environment::define`
  precisa de `String` (dono), porque a chave vai ficar guardada no
  `HashMap` por muito mais tempo do que a chamada de `eval_stmt` dura — daí
  `.clone()`/`.to_string()`. A confusão inicial do aluno ("`&` = ponteiro,
  sem `&` = dado direto na memória") foi corrigida: `String`/`Box`/`Vec` já
  são ponteiros para o heap mesmo sem `&`; a pergunta certa é "quem precisa
  continuar sendo dono depois que esta função terminar?", não "isso é um
  ponteiro?".

## Implementação

`aurora-interpreter/src/lib.rs`:

```rust
pub struct Environment {
    valores: HashMap<String, Value>,
    pai: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self { ... }                                   // sem pai
    pub fn get(&self, name: &str) -> Option<Value> { ... }         // lê, sobe a cadeia
    pub fn define(&mut self, name: String, value: Value) { ... }   // grava sempre, só aqui
    pub fn assign(&mut self, name: &str, value: Value)
        -> Result<(), String> { ... }                              // escreve onde já existe, senão Err
}
```

`eval_expr`, `eval_stmt` e `run_program` migrados de
`&HashMap<String, Value>` / `&mut HashMap<String, Value>` para
`&Environment` / `&mut Environment`. `Stmt::Assign` passou a delegar
inteiramente para `env.assign(name, valor)?` (a checagem `contains_key` +
`insert` manuais que existiam desde a Aula 07 saíram do `eval_stmt` e
viraram responsabilidade do próprio `Environment`); `Stmt::Let` passou a
usar `env.define(name.clone(), valor)`.

`aurora-cli/src/main.rs` trocou `HashMap::new()` por `Environment::new()`.
Testes trocaram `HashMap::new()` + `.insert()`/`.get()` diretos por
`Environment::new()` + `.define()`/`.get()` (que devolve `Option<Value>` já
clonado, não `Option<&Value>` — um dos ajustes exigidos pelos testes).

`aurora-parser/src/parser.rs`: método `is_at_end` (nunca usado) comentado
pelo aluno para eliminar o warning `dead_code` que aparecia ao compilar o
workspace.

Nenhuma sintaxe da Aurora cria escopo filho ainda — `run_program` continua
usando um único `Environment` global. É infraestrutura pura, sem efeito
observável em programas `.aur` até blocos/funções existirem.

Workspace compila sem warnings; 10 testes passando (7 em
`aurora-interpreter`, 3 em `aurora-parser`).

## Desafio

Duas peças de design/implementação, nessa ordem:

1. Modelo de escopos (`Option<Box<Environment>>` em cadeia pai-filho) —
   decisão do aluno, feita antes de qualquer código.
2. Migrar `eval_expr`/`eval_stmt`/`run_program`/`main.rs`/testes de
   `HashMap` cru para `Environment`, mantendo o mesmo comportamento externo
   (`let`/`=`/`echo` continuam funcionando exatamente como na Aula 07).

## Resolução (percurso do aluno)

**Design do modelo de escopos.** O aluno propôs a cadeia pai-filho
corretamente, mas descreveu o efeito ao contrário na primeira tentativa
(achou que escopo léxico significava "de fora não alcança de dentro sem
passar por parâmetro"; na verdade é o oposto — o escopo onde uma função foi
*definida* é alcançado automaticamente via a cadeia, o que não seria
verdade se a busca dependesse de onde ela foi *chamada*). Corrigido em
conversa antes de escrever `struct Environment`.

**`Environment::new()` — confusão `None` vs. `()`.** Primeira tentativa
usou `pai: ()` e importou `std::ptr::null`, misturando o "nenhum valor de
tipo T" do Rust (`Option::None`) com ponteiro nulo/`null` de PHP. Corrigido
depois de nomear a diferença explicitamente.

**`get` — bloqueio real.** No `Option<Box<Self>>` + recursão, o aluno
declarou "estou querendo desistir de tudo. Está muito complicado" — sinal
mais forte que um "estou perdido" comum. Resposta: pausar o avanço de
código, validar que esse é um dos pontos mais difíceis de onboarding em
Rust mesmo para quem já tem experiência, listar vitórias concretas já
alcançadas na própria sessão, escrever `get` inteiro comentado linha a
linha (reaproveitando a analogia já conhecida de `soma_todos` numa lista
ligada), e confirmar como o aluno estava se sentindo antes de seguir. A
pedido dele, a aula foi pausada ali e retomada depois.

**Recap unificador.** Ainda na mesma sessão anterior, o aluno fez uma
observação própria — "match parece um coringa pra quase tudo" — respondida
com uma tabela lado a lado mapeando `Option`, `Result`, `Value`, `Token`,
`Expr`, `Stmt` e `Environment.pai` à mesma pergunta ("qual desses formatos
é esse valor?"). Funcionou como reforço de confiança pós-bloqueio, não
conteúdo novo.

**Retomada desta sessão — `assign`.** Ao voltar, o estado do arquivo foi
conferido contra a memória da pausa (batia exatamente: `struct`, `get` e
`define` prontos, `assign` pendente). Antes de escrever código, foi
colocada uma pergunta de design em duas partes (assinatura de `assign` —
`&mut self`, `Result`; e como fazer recursão *mutável* através de
`Option<Box<Environment>>`). O aluno respondeu "sinceramente eu não sei
responder" — sinal de bloqueio direto, mesmo sem o tom de frustração da
Aula 08 anterior — e `assign` foi escrito inteiro comentado, espelhando a
estrutura do `get` já existente e destacando a peça nova
(`self.pai.as_deref_mut()` em vez de `match &self.pai`).

**Migração `HashMap` → `Environment`.** Para essa parte o aluno pediu só
dicas antes de tentar sozinho — recebeu uma lista de 7 pontos (assinaturas,
a pegadinha do `.clone()` que deixa de ser necessário em `Expr::Ident`
porque `Environment::get` já devolve `Option<Value>` em vez de
`Option<&Value>`, consolidar `Stmt::Assign`/`Stmt::Let` nos métodos novos,
o campo `valores` privado, `main.rs`, testes) e tentou por conta própria.
Travou num erro real de tipo (`env.define(name, valor)` esperando `String`,
recebendo `&String`, `lib.rs:179`) — resolvido lendo a mensagem do
compilador e relacionando com o `.clone()` que o código antigo já usava no
mesmo lugar antes de existir `Environment`.

Nesse ponto o aluno pediu uma explicação mais ampla de `&` (referência) vs.
posse, formulando a própria hipótese ("tem `&` = ponteiro/endereço na
memória; sem `&` = o dado direto") enquanto terminava as correções. A
hipótese foi corrigida (tipos como `String`/`Box`/`Vec` já são ponteiros
para o heap mesmo sem `&`; o que `&` de fato marca é *empréstimo* — "quem
continua sendo dono depois que a função termina", não "é ou não é
ponteiro") com um diagrama de memória (stack/heap) comparando `String` e
`&String` lado a lado.

**Teste com bug real.** Rodando o workspace depois das correções do aluno:
compilava limpo (inclusive sem o warning de `is_at_end`, que o aluno já
tinha comentado por conta própria), mas 1 dos 7 testes de
`aurora-interpreter` falhava — `eval_ident_busca_no_environment` chamava
`env.assign("x", Value::Int(3))?` para *preparar* o cenário do teste, mas
`assign` exige que a variável já exista (é a mesma regra de "reatribuição
só depois de `let`" da Aula 07) — `x` nunca tinha sido declarada, então
devolvia `Err` e o teste falhava. Isso não era um bug no `Environment`; era
o teste chamando o método errado (`assign` em vez de `define`) para o que
queria fazer. Apontado como pergunta primeiro ("qual dos dois é pra criar,
qual é só pra atualizar?"); o aluno declarou bloqueio ("não consegui
desvendar o mistério") e a correção foi feita e explicada diretamente:
`env.define("x".to_string(), Value::Int(3))`, sem `?` (já que `define`
devolve `()`, não `Result`).

## Conclusão

`Environment` agora suporta uma cadeia de escopos pai-filho
(`get`/`define`/`assign`), e todo o interpretador (`eval_expr`,
`eval_stmt`, `run_program`, `aurora-cli`, testes) foi migrado do
`HashMap<String, Value>` cru para essa estrutura, sem mudar o
comportamento observável de nenhum programa Aurora existente. Nenhuma
sintaxe ainda cria um escopo filho de verdade — isso só vai acontecer
quando blocos/funções existirem.

## Próximos passos

Aula 09 — Strings: primeiro tipo de dado novo desde `Int`/`Float`. Escopo
filho de verdade (`pai: Some(...)`) só vai aparecer como efeito colateral
de sintaxe real nas Aulas 11 (Condições/blocos) e 13 (Funções).
