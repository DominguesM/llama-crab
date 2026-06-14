import { Channel, invoke } from "@tauri-apps/api/core"
import { LlamaCrabTauri } from "@llama-crab/tauri"
import "./styles.css"

const MODEL_REPO = "LiquidAI/LFM2.5-350M-GGUF"
const MODEL_FILE = "LFM2.5-350M-Q4_K_M.gguf"
const MODEL_ID = "lfm2.5-350m-q4-k-m"

const client = new LlamaCrabTauri()

type DownloadProgress = {
  state: "cached" | "started" | "downloading" | "finished"
  downloaded_bytes: number
  total_bytes?: number | null
  path?: string | null
}

let modelReady = false
let loadingModel: Promise<void> | null = null

const form = document.querySelector<HTMLFormElement>("#chat-form")
const input = document.querySelector<HTMLTextAreaElement>("#prompt")
const answer = document.querySelector<HTMLElement>("#answer")
const status = document.querySelector<HTMLElement>("#status")
const sendButton = document.querySelector<HTMLButtonElement>("#send-button")
const clearButton = document.querySelector<HTMLButtonElement>("#clear-button")

function setStatus(text: string) {
  if (status) {
    status.textContent = text
  }
}

function setBusy(isBusy: boolean) {
  if (sendButton) {
    sendButton.disabled = isBusy
  }
  if (input) {
    input.disabled = isBusy
  }
}

function formatBytes(value: number) {
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`
  }
  return `${(value / 1024 / 1024).toFixed(1)} MB`
}

function describeDownloadProgress(progress: DownloadProgress) {
  if (progress.state === "cached") {
    return `Modelo ja baixado (${formatBytes(progress.downloaded_bytes)}).`
  }
  if (progress.state === "finished") {
    return "Download concluido. Carregando o modelo local..."
  }
  if (!progress.total_bytes) {
    return `Baixando modelo: ${formatBytes(progress.downloaded_bytes)}`
  }

  const percent = Math.min(100, Math.round((progress.downloaded_bytes / progress.total_bytes) * 100))
  return `Baixando modelo: ${formatBytes(progress.downloaded_bytes)} / ${formatBytes(progress.total_bytes)} (${percent}%)`
}

function clearChat() {
  if (input) {
    input.value = ""
    input.focus()
  }
  if (answer) {
    answer.textContent = ""
  }
  setStatus(`Modelo: ${MODEL_REPO} / ${MODEL_FILE}`)
}

async function ensureModel() {
  if (modelReady) {
    return
  }
  if (loadingModel) {
    return loadingModel
  }

  loadingModel = (async () => {
    const onProgress = new Channel<DownloadProgress>()
    onProgress.onmessage = (progress) => {
      setStatus(describeDownloadProgress(progress))
    }

    setStatus("Verificando modelo local...")
    const modelPath = await invoke<string>("ensure_lfm_model", { onProgress })
    setStatus("Carregando o modelo local...")
    await client.models.load({
      model: MODEL_ID,
      path: modelPath,
      kind: "chat",
      mobile_preset: "balanced",
      n_ctx: 2048,
    })
    modelReady = true
    setStatus("Modelo pronto.")
  })()

  try {
    await loadingModel
  } finally {
    loadingModel = null
  }
}

async function askModel(prompt: string) {
  await ensureModel()
  if (answer) {
    answer.textContent = ""
  }

  const stream = await client.chat.completions.create({
    model: MODEL_ID,
    messages: [{ role: "user", content: prompt }],
    stream: true,
    max_tokens: 256,
    temperature: 0.7,
    llama_crab: {
      template: "chatml",
      top_k: 40,
    },
  })

  setStatus("Gerando resposta...")
  for await (const chunk of stream) {
    const text = chunk.choices[0]?.delta.content ?? ""
    if (text && answer) {
      answer.textContent += text
    }
  }
  setStatus("Pronto.")
}

form?.addEventListener("submit", async (event) => {
  event.preventDefault()
  const prompt = input?.value.trim() ?? ""
  if (!prompt) {
    input?.focus()
    return
  }

  setBusy(true)
  try {
    await askModel(prompt)
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    setStatus(`Erro: ${message}`)
  } finally {
    setBusy(false)
  }
})

clearButton?.addEventListener("click", clearChat)
clearChat()
