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
  // no live rendering; only render when Pixelate is pressed

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

  function markDirty() {
    upscaledDataURL = null;
    baseDataURL = null;
    if (outputEmpty) outputEmpty.textContent = "Press Pixelate to update preview";
    setPreview(null);
    updateButtons();
  }

  function updateButtons() {
    // Allow Pixelate click to prompt for file when no image
    enable(btnGen, true);
    enable(btnUpscaled, !!upscaledDataURL);
    enable(btnBase, !!selectedImage);
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
      // Let Rust parse grid string like "32" or "384x192"
      const val = gridSel.value.trim();
      const req = {
        image_data_url: selectedImage,
        grid_width: 0,
        grid_height: 0,
        grid_value: val,
        algorithm: algoSel.value,
        palette_name: paletteSel.value,
        tone_gamma: tone ? Number(tone.value) : undefined,
        denoise_sigma: denoise ? Number(denoise.value) : undefined,
      };
      const up = (await invoke("render_preview", { req })) as string;
      if (mySeq !== renderCounter) return; // stale
      upscaledDataURL = up;
      setPreview(upscaledDataURL);
    } catch (err) {
      console.error(err);
      showError(String(err));
    } finally {
      if (renderCounter === mySeq) enable(btnGen, true);
      updateButtons();
    }
  }

  // Live auto-render disabled per request; only render on Pixelate button press

  function handleFile(file: File) {
    const reader = new FileReader();
    reader.onload = (e) => {
      selectedImage = String(e.target?.result || "");
      if (thumb && dropHint) {
        thumb.src = selectedImage;
        thumb.style.display = "";
        dropHint.style.display = "none";
      }
      markDirty();
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
    (async () => {
      try {
        if (!selectedImage) return;
        if (!baseDataURL) {
          const val = gridSel?.value?.trim() || "32";
          const req = {
            image_data_url: selectedImage,
            grid_width: 0,
            grid_height: 0,
            grid_value: val,
            algorithm: algoSel?.value || "Standard",
            palette_name: paletteSel?.value || undefined,
            tone_gamma: tone ? Number(tone.value) : undefined,
            denoise_sigma: denoise ? Number(denoise.value) : undefined,
          };
          baseDataURL = (await invoke("render_base", { req })) as string;
        }
        if (baseDataURL) downloadDataURL(baseDataURL, "bitcrush-base.png");
      } catch (err) {
        console.error(err);
      }
    })();
  });

  // Mark dirty on control changes, but do not auto-render
  paletteSel?.addEventListener("change", markDirty);
  algoSel?.addEventListener("change", markDirty);
  gridSel?.addEventListener("change", markDirty);
  tone?.addEventListener("input", markDirty);
  denoise?.addEventListener("input", markDirty);
});
