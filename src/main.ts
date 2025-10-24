import { invoke } from "@tauri-apps/api/core";
import { mountRoutes } from "./app/router";

type PaletteTuple = [string, number[][]];

const qs = <T extends HTMLElement>(sel: string) => document.querySelector(sel) as T | null;

/**
 * Loads an image file and returns a properly-oriented data URL.
 * This fixes issues where images with EXIF orientation metadata appear rotated.
 */
async function loadImageWithCorrectOrientation(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      const dataUrl = String(e.target?.result || "");
      const img = new Image();
      img.onload = () => {
        // Create canvas and draw image (browsers auto-apply EXIF orientation)
        const canvas = document.createElement("canvas");
        canvas.width = img.naturalWidth;
        canvas.height = img.naturalHeight;
        const ctx = canvas.getContext("2d");
        if (!ctx) {
          reject(new Error("Could not get canvas context"));
          return;
        }
        ctx.drawImage(img, 0, 0);
        // Convert back to data URL with correct orientation
        resolve(canvas.toDataURL("image/png"));
      };
      img.onerror = () => reject(new Error("Failed to load image"));
      img.src = dataUrl;
    };
    reader.onerror = () => reject(new Error("Failed to read file"));
    reader.readAsDataURL(file);
  });
}

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

function flashDownload(el: HTMLButtonElement) {
  const prev = el.textContent;
  const { width } = el.getBoundingClientRect();
  el.style.width = `${width}px`;
  el.textContent = "Saved!";
  setTimeout(() => {
    el.textContent = prev;
    el.style.width = "";
  }, 1200);
}

async function loadPalettes() {
  const list = (await invoke("list_palettes")) as PaletteTuple[];
  const sel = qs<HTMLSelectElement>("#palette");
  if (!sel) return list;
  // sort by color count ascending
  const sorted = [...list].sort((a, b) => (a[1]?.length || 0) - (b[1]?.length || 0));
  sel.innerHTML = "";
  for (const [name] of sorted) {
    const opt = document.createElement("option");
    opt.textContent = name;
    sel.appendChild(opt);
  }
  return sorted;
}

window.addEventListener("DOMContentLoaded", async () => {
  // Set up simple in-app routing between Pixelizer and Filters pages
  mountRoutes();
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
  const preContrast = qs<HTMLInputElement>("#preContrast");
  const preContrastLabel = qs<HTMLDivElement>("#preContrastLabel");
  const preSaturation = qs<HTMLInputElement>("#preSaturation");
  const preSaturationLabel = qs<HTMLDivElement>("#preSaturationLabel");
  const preHue = qs<HTMLInputElement>("#preHue");
  const preHueLabel = qs<HTMLDivElement>("#preHueLabel");
  const btnGen = qs<HTMLButtonElement>("#generate");
  const btnUpscaled = qs<HTMLButtonElement>("#download-upscaled");
  const btnBase = qs<HTMLButtonElement>("#download-base");
  const optNightVision = qs<HTMLInputElement>("#optNightVision");
  const optAddBlack = qs<HTMLInputElement>("#optAddBlack");
  const optAddWhite = qs<HTMLInputElement>("#optAddWhite");
  const optInvert = qs<HTMLInputElement>("#optInvert");
  const exportScale = qs<HTMLSelectElement>("#exportScale");
  const exportFilter = qs<HTMLSelectElement>("#exportFilter");
  const exportClamp = qs<HTMLInputElement>("#exportClamp");

  // Filters page elements
  const fDropzone = qs<HTMLDivElement>("#filters-dropzone");
  const fFileInput = qs<HTMLInputElement>("#filters-file");
  const fThumb = qs<HTMLImageElement>("#filters-thumb");
  const fDropHint = qs<HTMLDivElement>("#filters-dropHint");
  const fOutput = qs<HTMLImageElement>("#filters-output");
  const fOutputEmpty = qs<HTMLDivElement>("#filters-outputEmpty");
  const fBtnGen = qs<HTMLButtonElement>("#filters-generate");
  const fKind = qs<HTMLSelectElement>("#filters-kind");

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
      outputEmpty.style.display = "none";
    }
  }

  function showError(_message: string) {
    if (!output || !outputEmpty) return;
    output.style.display = "none";
    output.src = "";
    outputEmpty.style.display = "none";
  }

  function markDirty() {
    // Clear cached outputs; keep current image until re-render
    upscaledDataURL = null;
    baseDataURL = null;
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
  function updatePreContrastLabel() {
    if (preContrast && preContrastLabel) preContrastLabel.textContent = `Contrast: ${Number(preContrast.value).toFixed(2)}`;
  }
  function updatePreSaturationLabel() {
    if (preSaturation && preSaturationLabel) preSaturationLabel.textContent = `Saturation: ${Number(preSaturation.value).toFixed(2)}`;
  }
  function updatePreHueLabel() {
    if (preHue && preHueLabel) preHueLabel.textContent = `Hue: ${Number(preHue.value).toFixed(0)}°`;
  }
  updateToneLabel();
  updateDenoiseLabel();
  updatePreContrastLabel();
  updatePreSaturationLabel();
  updatePreHueLabel();
  tone?.addEventListener("input", updateToneLabel);
  denoise?.addEventListener("input", updateDenoiseLabel);
  preContrast?.addEventListener("input", updatePreContrastLabel);
  preSaturation?.addEventListener("input", updatePreSaturationLabel);
  preHue?.addEventListener("input", updatePreHueLabel);

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
        display_size: 1000,
        tone_gamma: tone ? Number(tone.value) : undefined,
        denoise_sigma: denoise ? Number(denoise.value) : undefined,
        pre_contrast: preContrast ? Number(preContrast.value) : undefined,
        pre_saturation: preSaturation ? Number(preSaturation.value) : undefined,
        pre_hue_degrees: preHue ? Number(preHue.value) : undefined,
        add_black_to_palette: optAddBlack?.checked || false,
        add_white_to_palette: optAddWhite?.checked || false,
        invert_colors: optInvert?.checked || false,
        night_vision_prefilter: optNightVision?.checked || false,
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

  async function handleFile(file: File) {
    try {
      selectedImage = await loadImageWithCorrectOrientation(file);
      if (thumb && dropHint) {
        thumb.src = selectedImage;
        thumb.style.display = "";
        dropHint.style.display = "none";
      }
      // New image selected: clear preview so user knows to render with new settings
      markDirty();
      setPreview(null);
    } catch (err) {
      console.error("Failed to load image:", err);
      showError("Failed to load image");
    }
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
    (async () => {
      try {
        if (!selectedImage || !algoSel || !gridSel || !paletteSel) return;
        const val = gridSel.value.trim();
        const req = {
          image_data_url: selectedImage,
          grid_width: 0,
          grid_height: 0,
          grid_value: val,
          algorithm: algoSel.value,
          palette_name: paletteSel.value,
          display_size: 2000,
          tone_gamma: tone ? Number(tone.value) : undefined,
          denoise_sigma: denoise ? Number(denoise.value) : undefined,
          pre_contrast: preContrast ? Number(preContrast.value) : undefined,
          pre_saturation: preSaturation ? Number(preSaturation.value) : undefined,
          pre_hue_degrees: preHue ? Number(preHue.value) : undefined,
          add_black_to_palette: optAddBlack?.checked || false,
          add_white_to_palette: optAddWhite?.checked || false,
          invert_colors: optInvert?.checked || false,
          night_vision_prefilter: optNightVision?.checked || false,
          export_scale_factor: exportScale ? Number(exportScale.value) : 2,
          export_filter: exportFilter?.value || "nearest",
          export_clamp_2000: exportClamp?.checked ?? true,
        };
        const hi = (await invoke("render_preview", { req })) as string;
        downloadDataURL(hi, "bitcrush-preview-2000.png");
        flashDownload(el);
      } catch (err) {
        console.error(err);
      }
    })();
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
            pre_contrast: preContrast ? Number(preContrast.value) : undefined,
            pre_saturation: preSaturation ? Number(preSaturation.value) : undefined,
            pre_hue_degrees: preHue ? Number(preHue.value) : undefined,
          };
          baseDataURL = (await invoke("render_base", { req })) as string;
        }
        if (baseDataURL) {
          downloadDataURL(baseDataURL, "bitcrush-base.png");
          flashDownload(el);
        }
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
  optAddBlack?.addEventListener("change", markDirty);
  optAddWhite?.addEventListener("change", markDirty);
  optInvert?.addEventListener("change", markDirty);
  optNightVision?.addEventListener("change", markDirty);

  // ---------------- Filters page wiring ----------------
  let filtersImage: string | null = null;

  function filtersSetPreview(src: string | null) {
    if (!fOutput || !fOutputEmpty) return;
    if (src) {
      fOutput.style.display = "";
      fOutput.src = src;
      fOutputEmpty.style.display = "none";
    } else {
      fOutput.style.display = "none";
      fOutput.src = "";
      fOutputEmpty.style.display = "none";
    }
  }

  function filtersShowError(_message: string) {
    if (!fOutput || !fOutputEmpty) return;
    fOutput.style.display = "none";
    fOutput.src = "";
    fOutputEmpty.style.display = "none";
  }

  async function filtersHandleFile(file: File) {
    try {
      filtersImage = await loadImageWithCorrectOrientation(file);
      if (fThumb && fDropHint) {
        fThumb.src = filtersImage;
        fThumb.style.display = "";
        fDropHint.style.display = "none";
      }
      filtersSetPreview(null);
    } catch (err) {
      console.error("Failed to load image:", err);
      filtersShowError("Failed to load image");
    }
  }

  async function renderFiltersNow() {
    if (!filtersImage) return;
    try {
      const kind = (fKind?.value || "VHS").trim();
      const req = {
        image_data_url: filtersImage,
        display_size: 1000,
        steps: [{ name: kind, amount: 1.0, enabled: true }],
      };
      const up = (await invoke("render_filters_chain_preview", { req })) as string;
      filtersSetPreview(up);
    } catch (err) {
      console.error(err);
      filtersShowError(String(err));
    }
  }

  fDropzone?.addEventListener("click", () => fFileInput?.click());
  fDropzone?.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") fFileInput?.click();
  });
  fDropzone?.addEventListener("dragover", (e) => e.preventDefault());
  fDropzone?.addEventListener("drop", (e) => {
    e.preventDefault();
    const f = e.dataTransfer?.files?.[0];
    if (f) filtersHandleFile(f);
  });
  fFileInput?.addEventListener("change", (e) => {
    const t = e.target as HTMLInputElement;
    const f = t.files?.[0];
    if (f) filtersHandleFile(f);
  });
  fBtnGen?.addEventListener("click", async () => {
    if (!filtersImage) {
      fFileInput?.click();
      return;
    }
    await renderFiltersNow();
  });
});
