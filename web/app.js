const conversions = { md: ["pdf"], pdf: ["md"], docx: ["pdf", "md"] };
const labels = { md: "Markdown", pdf: "PDF", docx: "Word" };
const extensions = { md: ".md", pdf: ".pdf", docx: ".docx" };
const fileInput = document.querySelector("#file-input");
const dropzone = document.querySelector("#dropzone");
const dropContent = document.querySelector("#drop-content");
const selectedFile = document.querySelector("#selected-file");
const fromSelect = document.querySelector("#from-format");
const toSelect = document.querySelector("#to-format");
const convertButton = document.querySelector("#convert-button");
const formatNote = document.querySelector("#format-note");
const loadingCard = document.querySelector("#loading-card");
let file = null;

function targetsFor(source) {
  return conversions[source] || [];
}

function updateTargets() {
  const targets = targetsFor(fromSelect.value);
  const previous = toSelect.value;
  toSelect.replaceChildren(...targets.map((format) => {
    const option = document.createElement("option");
    option.value = format;
    option.textContent = `${labels[format]}  ${extensions[format]}`;
    return option;
  }));
  if (targets.includes(previous)) toSelect.value = previous;
  toSelect.disabled = targets.length === 0;
  convertButton.disabled = !file || targets.length === 0;
  if (targets.length === 0) {
    formatNote.textContent = "No conversion is available from this format.";
  } else if (file && file.name.toLowerCase().endsWith(extensions[fromSelect.value])) {
    formatNote.textContent = `${labels[fromSelect.value]} to ${targets.map((target) => labels[target]).join(" or ")}.`;
  } else if (file) {
    formatNote.textContent = `This file is ${file.name.split(".").pop().toUpperCase()}, but “from” is ${labels[fromSelect.value]}.`;
  } else {
    formatNote.textContent = "Choose the file type you’re starting with.";
  }
}

function setFile(nextFile) {
  if (!nextFile) return;
  const extension = `.${nextFile.name.split(".").pop().toLowerCase()}`;
  const detected = Object.keys(extensions).find((format) => extensions[format] === extension);
  if (!detected) {
    formatNote.textContent = "That file type isn’t supported yet. Try MD, PDF or DOCX.";
    return;
  }
  file = nextFile;
  fromSelect.value = detected;
  document.querySelector("#file-name").textContent = nextFile.name;
  document.querySelector("#file-size").textContent = `${(nextFile.size / 1024).toFixed(nextFile.size < 1024 * 1024 ? 0 : 1)} ${nextFile.size < 1024 * 1024 ? "KB" : "MB"}`;
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
  if (!file.name.toLowerCase().endsWith(extensions[fromSelect.value])) {
    formatNote.textContent = `Please choose a ${labels[fromSelect.value]} file to match “from”.`;
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
    const filename = disposition.match(/filename="?([^";]+)"?/i)?.[1] || `converted${extensions[toSelect.value]}`;
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
  }
});

updateTargets();
