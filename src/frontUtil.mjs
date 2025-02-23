export function addLog(msg, type) {
    document.getElementById("s-logs").innerHTML +=
        `<div class="log-${type}">${msg}</div>`;
}

export function addFile(id, name, size) {
    let file = document.createElement("div");
    file.classList.add("d-file");
    {
        let detail = document.createElement("div");
        detail.classList.add("file-detail");

        let fileName = document.createElement("p");
        fileName.classList.add("file-name");
        fileName.textContent = name;
        detail.appendChild(fileName);

        let fileSize = document.createElement("p");
        fileSize.classList.add("file-size");
        fileSize.textContent = size;
        detail.appendChild(fileSize);

        file.appendChild(detail);
    }
    let ptext = document.createElement("p");
    ptext.classList.add("file-ptext");
    ptext.id = "ptext-" + id;
    file.appendChild(ptext);
    {
        let fileBar = document.createElement("div");
        fileBar.classList.add("file-bar");
        let progressBar = document.createElement("div");
        progressBar.classList.add("file-progress");
        progressBar.id = "progress-" + id;
        fileBar.appendChild(progressBar);
        file.appendChild(fileBar);
    }
    let pnum = document.createElement("p");
    pnum.classList.add("file-pnum");
    pnum.id = "pnum-" + id;
    pnum.textContent = "0%";
    file.appendChild(pnum);
    document.getElementById("s-files").appendChild(file);
}

export function clearLog() {
    document.getElementById("s-logs").childNodes.forEach(n => n.remove());
    document.getElementById("s-files").childNodes.forEach(n => n.remove());
}

export function fileProgress(id, progress) {
    document.getElementById("progress-" + id).style.cssText = "width:" + progress + "%";
    document.getElementById("pnum-" + id).textContent = progress + "%";
}

export function fileMessage(id, message) {
    document.getElementById("ptext-" + id).textContent = message;
}

export function addZip(id, name, count) {
    let file = document.createElement("div");
    file.classList.add("d-file");
    {
        let detail = document.createElement("div");
        detail.classList.add("file-detail");

        let fileName = document.createElement("p");
        fileName.classList.add("file-name");
        fileName.textContent = name;
        detail.appendChild(fileName);
    }
    let ptext = document.createElement("p");
    ptext.classList.add("file-ptext");
    ptext.id = "ptext-" + id;
    ptext.textContent = "Extraction started";
    file.appendChild(ptext);
    {
        let fileBar = document.createElement("div");
        fileBar.classList.add("file-bar");
        let progressBar = document.createElement("div");
        progressBar.classList.add("file-progress");
        progressBar.id = "progress-" + id;
        fileBar.appendChild(progressBar);
        file.appendChild(fileBar);
    }
    let pnum = document.createElement("p");
    pnum.classList.add("file-pnum");
    pnum.id = "pnum-" + id;
    pnum.textContent = "0/" + count;
    file.appendChild(pnum);
    document.getElementById("s-files").appendChild(file);
}

export function zipProgress(id, name, count, progress, now) {
    document.getElementById("progress-" + id).style.cssText = "width:" + progress + "%";
    document.getElementById("pnum-" + id).textContent = now + "/" + count;
    document.getElementById("ptext-" + id).textContent = "Extracted " + name;
}

export function stopElem(id) {
    document.getElementById(id).classList.remove("failed", "succeed", "ongoing");
}

/**
 * @param {Object} obj 
 * @param {string[]} musts 
 * @param {string} pref
 */
export function checkMust(obj, musts, pref) {
    if (!(musts.every(k => Object.keys(obj).includes(k)))) {
        addLog(pref + "プロパティが不足しています。必ず" + musts.map(p => "〝" + p + "〟").join("") + "が存在している必要があります。", "error");
        stop();
        return true;
    }
    return false;
}