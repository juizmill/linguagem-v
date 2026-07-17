# Instruções para a IA neste repositório

Este é o Projeto Aurora. Leia `MASTER.md` primeiro — ele é a referência
oficial de objetivo, papel da IA, ordem das aulas e regras do curso.

## Regra de documentação

Toda aula (ou trecho relevante de aprendizado, mesmo fora do formato de aula
completa) deve ser documentada em Markdown dentro de `aulas/NN-nome/README.md`,
seguindo a estrutura definida no MASTER.md (Objetivo, Conceitos, Implementação,
Desafio, Conclusão, Próximos passos).

Isso inclui não só o conteúdo previamente preparado, mas também o que
efetivamente foi discutido e resolvido em conversa com o aluno — perguntas
feitas, respostas dele, correções de raciocínio — para que o arquivo sirva
como material de revisão fiel ao que ele aprendeu, e não só um roteiro
genérico. Atualize o arquivo da aula ao final da conversa/sessão, não deixe
para depois.

## Regra de commit e PR ao final de cada aula

Ao final de cada aula (depois de documentar o `README.md` da aula, conforme
a regra acima), a IA deve:

1. Fazer commit de todas as alterações pendentes (código + documentação da
   aula).
2. Criar um Pull Request com essas mudanças.

Só depois disso a aula seguinte deve começar. Nunca acumular trabalho de
mais de uma aula sem commit/PR, e nunca iniciar a próxima aula com
alterações da aula anterior ainda pendentes.

## Regra do README do projeto

Sempre que uma alteração tornar o `README.md` da raiz do projeto
desatualizado (nova sintaxe suportada, mudança de comando de build/teste,
nova extensão de arquivo, nova estrutura de crates, etc.), atualizar o
`README.md` junto com essa alteração — não deixar para uma aula futura.
