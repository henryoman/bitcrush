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
  const btnGen = qs<HTMLButtonElement>("#generate");
  const btnUpscaled = qs<HTMLButtonElement>("#download-upscaled");
  const btnBase = qs<HTMLButtonElement>("#download-base");

  let selectedImage: string | null = null;
  let upscaledDataURL: string | null = null;
  let baseDataURL: string | null = null;

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

  function updateButtons() {
    // Allow Pixelate click to prompt for file when no image
    enable(btnGen, true);
    enable(btnUpscaled, !!upscaledDataURL);
    enable(btnBase, !!baseDataURL);
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
    if (!algoSel || !gridSel || !paletteSel) return;
    enable(btnGen, false);
    try {
      const displaySize = 640; // UI preview target; Rust will snap to integer multiples
      const req = {
        image_data_url: selectedImage,
        grid_size: Number(gridSel.value),
        algorithm: algoSel.value,
        palette_name: paletteSel.value,
        display_size: displaySize,
      };
      const up = (await invoke("render_preview", { req })) as string;
      const base = (await invoke("render_base", { req })) as string;
      upscaledDataURL = up;
      baseDataURL = base;
      setPreview(upscaledDataURL);
    } catch (err) {
      console.error(err);
    } finally {
      enable(btnGen, true);
      updateButtons();
    }
  });

  btnUpscaled?.addEventListener("click", () => {
    if (upscaledDataURL) downloadDataURL(upscaledDataURL, "bitcrush-upscaled.png");
  });
  btnBase?.addEventListener("click", () => {
    if (baseDataURL) downloadDataURL(baseDataURL, "bitcrush-base.png");
  });
});
