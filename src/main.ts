import { invoke } from "@tauri-apps/api/core";

type PaletteTuple = [string, number[][]];

const qs = <T extends HTMLElement>(sel: string) => document.querySelector(sel) as T | null;

function enable(el: HTMLElement | null, on: boolean) {
  if (!el) return;
  (el as HTMLButtonElement).disabled = !on;
}

function downloadDataURL(dataURL: string, filename: string) {
  const a = document.createElement("a");
  a.href = dataURL;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
}

async function loadPalettes() {
  const list = (await invoke("list_palettes")) as PaletteTuple[];
  const sel = qs<HTMLSelectElement>("#palette");
  if (!sel) return list;
  sel.innerHTML = "";
  for (const [name] of list) {
    const opt = document.createElement("option");
    opt.textContent = name;
    sel.appendChild(opt);
  }
  return list;
}

window.addEventListener("DOMContentLoaded", async () => {
  // Ensure dragging works on overlay titlebar for all platforms
  const dropzone = qs<HTMLDivElement>("#dropzone");
  const fileInput = qs<HTMLInputElement>("#file");
  const thumb = qs<HTMLImageElement>("#thumb");
  const dropHint = qs<HTMLDivElement>("#dropHint");
  const output = qs<HTMLImageElement>("#output");
  const outputEmpty = qs<HTMLDivElement>("#outputEmpty");
  const paletteSel = qs<HTMLSelectElement>("#palette");
  const algoSel = qs<HTMLSelectElement>("#algorithm");
  const gridSel = qs<HTMLSelectElement>("#grid");
  const tone = qs<HTMLInputElement>("#tone");
  const toneLabel = qs<HTMLDivElement>("#toneLabel");
  const denoise = qs<HTMLInputElement>("#denoise");
  const denoiseLabel = qs<HTMLDivElement>("#denoiseLabel");
  const btnGen = qs<HTMLButtonElement>("#generate");
  const btnUpscaled = qs<HTMLButtonElement>("#download-upscaled");
  const btnBase = qs<HTMLButtonElement>("#download-base");

  let selectedImage: string | null = null;
  let upscaledDataURL: string | null = null;
  let baseDataURL: string | null = null;
  let renderCounter = 0; // sequence for stale response protection
  let debounceTimer: number | undefined;

  await loadPalettes();

  function setPreview(src: string | null) {
    if (!output || !outputEmpty) return;
    if (src) {
      output.style.display = "";
      output.src = src;
      outputEmpty.style.display = "none";
    } else {
      output.style.display = "none";
      output.src = "";
      outputEmpty.style.display = "";
    }
  }

  function showError(message: string) {
    if (!output || !outputEmpty) return;
    output.style.display = "none";
    output.src = "";
    outputEmpty.style.display = "";
    outputEmpty.textContent = `Error: ${message}`;
  }

  function updateButtons() {
    // Allow Pixelate click to prompt for file when no image
    enable(btnGen, true);
    enable(btnUpscaled, !!upscaledDataURL);
    enable(btnBase, !!baseDataURL);
  }

  function updateToneLabel() {
    if (tone && toneLabel) toneLabel.textContent = `Gamma: ${Number(tone.value).toFixed(2)}`;
  }
  function updateDenoiseLabel() {
    if (denoise && denoiseLabel) denoiseLabel.textContent = `Sigma: ${Number(denoise.value).toFixed(1)}`;
  }
  updateToneLabel();
  updateDenoiseLabel();
  tone?.addEventListener("input", updateToneLabel);
  denoise?.addEventListener("input", updateDenoiseLabel);

  async function renderNow() {
    if (!selectedImage || !algoSel || !gridSel || !paletteSel) return;
    const mySeq = ++renderCounter;
    enable(btnGen, false);
    try {
      const displaySize = 640; // UI preview target; Rust will snap to integer multiples
      // Parse grid selection as NxM or single number
      const val = gridSel.value.trim();
      const m = val.match(/^(\d+)(?:x(\d+))?$/i);
      const gridWidth = m ? Number(m[1]) : Number(val);
      const gridHeight = m && m[2] ? Number(m[2]) : gridWidth;
      const req = {
        image_data_url: selectedImage,
        grid_width: gridWidth,
        grid_height: gridHeight,
        algorithm: algoSel.value,
        palette_name: paletteSel.value,
        display_size: displaySize,
        tone_gamma: tone ? Number(tone.value) : undefined,
        denoise_sigma: denoise ? Number(denoise.value) : undefined,
      };
      const up = (await invoke("render_preview", { req })) as string;
      const base = (await invoke("render_base", { req })) as string;
      if (mySeq !== renderCounter) return; // stale
      upscaledDataURL = up;
      baseDataURL = base;
      setPreview(upscaledDataURL);
    } catch (err) {
      console.error(err);
      showError(String(err));
    } finally {
      if (renderCounter === mySeq) enable(btnGen, true);
      updateButtons();
    }
  }

  function scheduleAutoRender() {
    if (!selectedImage) return;
    if (debounceTimer) window.clearTimeout(debounceTimer);
    debounceTimer = window.setTimeout(() => { void renderNow(); }, 180);
  }

  function handleFile(file: File) {
    const reader = new FileReader();
    reader.onload = (e) => {
      selectedImage = String(e.target?.result || "");
      if (thumb && dropHint) {
        thumb.src = selectedImage;
        thumb.style.display = "";
        dropHint.style.display = "none";
      }
      upscaledDataURL = null;
      baseDataURL = null;
      setPreview(null);
      updateButtons();
      scheduleAutoRender();
    };
    reader.readAsDataURL(file);
  }

  dropzone?.addEventListener("click", () => fileInput?.click());
  dropzone?.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") fileInput?.click();
  });
  dropzone?.addEventListener("dragover", (e) => e.preventDefault());
  dropzone?.addEventListener("drop", (e) => {
    e.preventDefault();
    const f = e.dataTransfer?.files?.[0];
    if (f) handleFile(f);
  });
  fileInput?.addEventListener("change", (e) => {
    const t = e.target as HTMLInputElement;
    const f = t.files?.[0];
    if (f) handleFile(f);
  });

  btnGen?.addEventListener("click", async () => {
    if (!selectedImage) {
      fileInput?.click();
      return;
    }
    await renderNow();
  });

  btnUpscaled?.addEventListener("click", (e) => {
    const el = e.currentTarget as HTMLButtonElement;
    el.classList.add("is-pressed");
    setTimeout(() => el.classList.remove("is-pressed"), 90);
    if (upscaledDataURL) downloadDataURL(upscaledDataURL, "bitcrush-upscaled.png");
  });
  btnBase?.addEventListener("click", (e) => {
    const el = e.currentTarget as HTMLButtonElement;
    el.classList.add("is-pressed");
    setTimeout(() => el.classList.remove("is-pressed"), 90);
    if (baseDataURL) downloadDataURL(baseDataURL, "bitcrush-base.png");
  });

  // Auto-render on control changes
  paletteSel?.addEventListener("change", scheduleAutoRender);
  algoSel?.addEventListener("change", scheduleAutoRender);
  gridSel?.addEventListener("change", scheduleAutoRender);
  tone?.addEventListener("input", scheduleAutoRender);
  denoise?.addEventListener("input", scheduleAutoRender);
});
