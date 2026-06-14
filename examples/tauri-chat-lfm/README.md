# tauri-chat-lfm

Exemplo Tauri v2 simples para conversar com um unico modelo local usando `tauri-plugin-llama-crab` e `@llama-crab/tauri`.

Modelo fixo:

- Repositorio: `LiquidAI/LFM2.5-350M-GGUF`
- Arquivo: `LFM2.5-350M-Q4_K_M.gguf`

O app faz download automatico do modelo na primeira mensagem caso o arquivo ainda nao exista no diretorio local de dados da aplicacao. Durante essa primeira execucao, a tela mostra o progresso do download em MB e percentual quando o tamanho total estiver disponivel.

## Executar

Na raiz do repositorio:

```sh
pnpm install
pnpm --filter tauri-chat-lfm tauri dev
```

Tambem e possivel executar pelo wrapper geral de exemplos:

```sh
./examples/run.sh tauri_chat_lfm
```

O primeiro envio pode demorar porque o app baixa aproximadamente 229 MB do Hugging Face antes de carregar o modelo.

## Interface

A tela tem apenas:

- campo de mensagem
- botao `Enviar`
- botao `Limpar`
- area da resposta atual

O exemplo nao mantem historico de conversa. Cada envio usa somente a mensagem digitada no momento.
