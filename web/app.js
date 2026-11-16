const fileInput = document.querySelector("#file-input");
const dropzone = document.querySelector("#dropzone");
const dropContent = document.querySelector("#drop-content");
const selectedFile = document.querySelector("#selected-file");
const fromSelect = document.querySelector("#from-format");
const toSelect = document.querySelector("#to-format");
const convertButton = document.querySelector("#convert-button");
const formatNote = document.querySelector("#format-note");
const loadingCard = document.querySelector("#loading-card");
const supportedTypes = document.querySelector("#supported-types");

let file = null;
let capabilities = { formats: [], format_details: {}, conversions: [] };
let conversionMap = new Map();
let extensionToFormat = new Map();

function formatSpec(name) {
  return capabilities.format_details?.[name] || null;
}

function formatLabel(name) {
  const spec = formatSpec(name);
  return spec?.label || name.toUpperCase();
}

function formatExtension(name) {
  const spec = formatSpec(name);
  return spec?.extension || `.${name}`;
}

function formatOptionLabel(name) {
  const spec = formatSpec(name);
  if (!spec) return name.toUpperCase();
  return `${spec.label}  ${spec.extension}`;
}

function targetsFor(source) {
  return conversionMap.get(source) || [];
}

function buildIndexes(data) {
  capabilities = data || { formats: [], format_details: {}, conversions: [] };
  conversionMap = new Map();
  extensionToFormat = new Map();

  for (const conversion of capabilities.conversions || []) {
    if (!conversionMap.has(conversion.from)) conversionMap.set(conversion.from, []);
    conversionMap.get(conversion.from).push(conversion.to);
  }

  for (const [name, spec] of Object.entries(capabilities.format_details || {})) {
    for (const ext of spec.extensions || [spec.extension || `.${name}`]) {
      if (ext) extensionToFormat.set(String(ext).toLowerCase(), name);
    }
  }
}

function populateFromSelect(preferred) {
  const formats = capabilities.formats || [];
  fromSelect.replaceChildren(...formats.map((name) => {
    const option = document.createElement("option");
    option.value = name;
    option.textContent = formatOptionLabel(name);
    return option;
  }));
  if (preferred && formats.includes(preferred)) {
    fromSelect.value = preferred;
  } else if (formats.length > 0) {
    fromSelect.value = formats[0];
  }
}

function updateAccept() {
  const extensions = [];
  for (const spec of Object.values(capabilities.format_details || {})) {
    for (const ext of spec.extensions || [spec.extension]) {
      if (ext) extensions.push(ext);
    }
  }
  fileInput.accept = [...new Set(extensions)].join(",");
}

function updateSupportedTypes() {
  const names = capabilities.formats || [];
  if (names.length === 0) {
    supportedTypes.textContent = "No formats available";
    return;
  }
  const display = names.map((name) => name.toUpperCase());
  supportedTypes.textContent = display.join(" · ");
}

function updateTargets() {
  const targets = targetsFor(fromSelect.value);
  const previous = toSelect.value;

  toSelect.replaceChildren(...targets.map((name) => {
    const option = document.createElement("option");
    option.value = name;
    option.textContent = formatOptionLabel(name);
    return option;
  }));

  if (targets.includes(previous)) {
    toSelect.value = previous;
  }

  toSelect.disabled = targets.length === 0;
  convertButton.disabled = !file || targets.length === 0 || fromSelect.disabled;

  if (capabilities.error && (capabilities.formats || []).length === 0) {
    formatNote.textContent = capabilities.error;
    return;
  }

  if (targets.length === 0) {
    formatNote.textContent = "No conversion is available from this format.";
    return;
  }

  if (file) {
    const fileExt = `.${file.name.split(".").pop().toLowerCase()}`;
    const detected = extensionToFormat.get(fileExt);
    if (detected && detected !== fromSelect.value) {
      formatNote.textContent = `This file looks like ${formatLabel(detected)}, but “from” is ${formatLabel(fromSelect.value)}.`;
      return;
    }
    formatNote.textContent = `${formatLabel(fromSelect.value)} to ${targets.map((target) => formatLabel(target)).join(" or ")}.`;
    return;
  }

  formatNote.textContent = "Choose the file type you’re starting with.";
}

function detectFormat(filename) {
  const ext = `.${filename.split(".").pop().toLowerCase()}`;
  return extensionToFormat.get(ext) || null;
}

function setFile(nextFile) {
  if (!nextFile) return;
  const detected = detectFormat(nextFile.name);
  if (!detected) {
    const supported = (capabilities.formats || []).map((name) => formatExtension(name)).join(", ");
    formatNote.textContent = supported
      ? `That file type isn’t supported yet. Try: ${supported}`
      : "That file type isn’t supported yet.";
    return;
  }

  file = nextFile;
  fromSelect.value = detected;
  document.querySelector("#file-name").textContent = nextFile.name;
  const kb = nextFile.size / 1024;
  const mb = kb / 1024;
  document.querySelector("#file-size").textContent = mb >= 1 ? `${mb.toFixed(1)} MB` : `${Math.max(1, Math.round(kb))} KB`;
  dropContent.hidden = true;
  selectedFile.hidden = false;
  updateTargets();
}

function clearFile(event) {
  event?.stopPropagation();
  file = null;
  fileInput.value = "";
  selectedFile.hidden = true;
  dropContent.hidden = false;
  updateTargets();
}

dropzone.addEventListener("click", (event) => {
  if (event.target.closest("button")) return;
  fileInput.click();
});

dropzone.addEventListener("keydown", (event) => {
  if (event.target === dropzone && (event.key === "Enter" || event.key === " ")) {
    event.preventDefault();
    fileInput.click();
  }
});

document.querySelector("#browse-button").addEventListener("click", (event) => {
  event.stopPropagation();
  fileInput.click();
});

document.querySelector("#remove-file").addEventListener("click", clearFile);
fileInput.addEventListener("change", () => setFile(fileInput.files[0]));
fromSelect.addEventListener("change", updateTargets);

for (const eventName of ["dragenter", "dragover"]) {
  dropzone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropzone.classList.add("is-over");
  });
}

for (const eventName of ["dragleave", "drop"]) {
  dropzone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropzone.classList.remove("is-over");
  });
}

dropzone.addEventListener("drop", (event) => setFile(event.dataTransfer.files[0]));

convertButton.addEventListener("click", async () => {
  if (!file) return;

  const detected = detectFormat(file.name);
  if (detected && detected !== fromSelect.value) {
    formatNote.textContent = `Please choose a ${formatLabel(fromSelect.value)} file to match “from”.`;
    return;
  }

  convertButton.disabled = true;
  loadingCard.hidden = false;
  document.querySelector("#loading-title").innerHTML = "Making a little<br />format magic.";
  document.querySelector("#loading-copy").textContent = "Your file is on its way.";
  loadingCard.scrollIntoView({ behavior: "smooth", block: "nearest" });

  const form = new FormData();
  form.append("file", file, file.name);
  form.append("from", fromSelect.value);
  form.append("to", toSelect.value);

  try {
    const response = await fetch("/api/convert", { method: "POST", body: form });
    const type = response.headers.get("content-type") || "";
    if (!response.ok || type.includes("application/json")) {
      const result = await response.json();
      throw new Error(result.error || "Conversion didn’t work. Please try again.");
    }

    const resultBlob = await response.blob();
    const disposition = response.headers.get("content-disposition") || "";
    const fallbackName = `${file.name.replace(/\.[^.]+$/, "") || "converted"}${formatExtension(toSelect.value)}`;
    const filename = disposition.match(/filename="?([^";]+)"?/i)?.[1] || fallbackName;

    const anchor = document.createElement("a");
    anchor.href = URL.createObjectURL(resultBlob);
    anchor.download = filename;
    anchor.click();
    URL.revokeObjectURL(anchor.href);

    document.querySelector("#loading-title").textContent = "All done. Nice.";
    document.querySelector("#loading-copy").textContent = `${filename} has been downloaded.`;
    formatNote.textContent = "Your converted file is ready.";
  } catch (error) {
    document.querySelector("#loading-title").textContent = "A small hiccup.";
    document.querySelector("#loading-copy").textContent = error.message;
    formatNote.textContent = error.message;
  } finally {
    convertButton.disabled = false;
    updateTargets();
  }
});

async function loadCapabilities() {
  try {
    const response = await fetch("/api/capabilities", { headers: { Accept: "application/json" } });
    const data = await response.json();
    if (!response.ok) {
      throw new Error(data.error || "Could not load supported formats.");
    }
    buildIndexes(data);
  } catch (error) {
    buildIndexes({
      ok: false,
      error: error.message,
      formats: [],
      format_details: {},
      conversions: [],
    });
    fromSelect.disabled = true;
    toSelect.disabled = true;
    convertButton.disabled = true;
  }

  populateFromSelect();
  updateAccept();
  updateSupportedTypes();
  updateTargets();
}

loadCapabilities();
