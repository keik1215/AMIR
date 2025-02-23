import { initFront } from "./frontHandler.mjs";
import { addFile, addLog, addZip, checkMust, clearLog, fileMessage, fileProgress, zipProgress, stopElem } from "./frontUtil.mjs";

const { ask, message, open } = window.__TAURI__.dialog;
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { openUrl } = window.__TAURI__.opener;

let noAuto = false;
let stopped = false;
let installConfig;


initFront();


listen("change", ev => {
  addLog(ev.payload.message, "info");
});

listen("newfile", ev => {
  console.log(ev)
  addFile(ev.payload.id, ev.payload.name, ev.payload.size);
  fileMessage("Downloading " + ev.payload.name);
});

listen("filechange", ev => {
  console.log(ev.payload);
  fileProgress(ev.payload.id, ev.payload.progress);
});

listen("errfile", ev => {
  document.getElementById("progress-" + ev.payload.id).classList.add("failed");
  document.getElementById("ptext-" + ev.payload.id).textContent = ev.payload.message;
  document.getElementById("p-down").classList.remove("ongoing");
  document.getElementById("p-down").classList.add("failed");
});

listen("zipstart", ev => {
  addZip(ev.payload.id, ev.payload.name, ev.payload.files);
});

listen("zipfile", ev => {
  zipProgress(ev.payload.id, ev.payload.name, ev.payload.count, ev.payload.progress, ev.payload.now);
});

listen("proferr", ev => {
  document.getElementById("p-make").classList.remove("ongoing");
  document.getElementById("p-make").classList.add("failed");
  addLog(ev.payload.message, "error");
});


document.getElementById("selectmc").addEventListener("click", async () => {
  try {
    const selected = await open({
      multiple: false,
      directory: true,
      defaultPath: (await invoke("app_data"))
    });
    if (selected && typeof selected === "string") {
      document.getElementById("dirmc").textContent = selected;
    }
  } catch (error) {
    console.error("Error has occured when read file: ", error);
  }
});
document.getElementById("selectcd").addEventListener("click", async () => {
  try {
    const selected = await open({
      multiple: false,
      directory: true,
      defaultPath: (await invoke("app_data"))
    });
    if (selected && typeof selected === "string") {
      document.getElementById("dircd").textContent = selected;
    }
  } catch (error) {
    console.error("Error has occured when read file: ", error);
  }
});

document.getElementById("startbut").addEventListener("click", () => {
  if (document.getElementById("startbut").textContent === "CANCEL INSTALLING") {
    stop();
  } else {
    if (!(document.getElementById("dircd").textContent != 0 && (noAuto || document.getElementById("dirmc").textContent))) {
      message(".minecraftディレクトリ、または構成ディレクトリが設定されていません。両方設定してから再度試してください。", { title: "Auto Mod Installer", kind: "error" });
    } else {
      document.getElementById("user-input").classList.add("disappear");
      document.getElementById("startbut").classList.add("cancel");
      document.getElementById("startbut").textContent = "CANCEL INSTALLING";
      fetch(document.getElementById("i-url").value)
        .then(res => {
          res.json().then(json => {
            start(json, noAuto ? null : document.getElementById("dirmc").textContent, document.getElementById("dircd").textContent);
          })
            .catch(e => {
              addLog("レスポンスをJSONに変換できませんでした。:" + e, "error");
            });
        })
        .catch(e => {
          addLog("URLを取得できませんでした。:" + e, "error");
        });
    }
  }
});

document.getElementById("auto-info").addEventListener("click", () => {
  noAuto = !noAuto;
});

/**
 * 
 * @param {{forgeVersion: string, name: String, mods: {type: string, url: string, name: string?}[], otherFiles: Object[]}} config 
 * @param {string} mcPath 
 * @param {string} dirPath 
 * @returns 
 */
async function start(config, mcPath, dirPath) {
  stopped = false;
  const pCheckConf = document.getElementById("p-chec");
  const pDiagnosis = document.getElementById("p-diag");
  const pDownload = document.getElementById("p-down");
  const pMakeprof = document.getElementById("p-make");
  clearLog();
  //1.Check Config
  {
    console.log(config)
    const version = config.forgeVersion.split("-");

    if (checkMust(config, ["mods", "name", "forgeVersion"], "構成")) return;

    if ((!config.forgeVersion) || version.length != 3 || version[1] != "forge") {
      addLog("構成プロパティ〝forgeVersion〟が正しくありません。〝\"forgeVersion\": \"1.20.1-forge-47.3.33\"〟このようなフォーマットに基づいている必要があります。", "error")
      stop();
      return;
    }

    for (let i = 0; i < config.mods.length; i++) {
      const elem = config.mods[i];
      if (!elem.type) {
        addLog("配列mods内" + i + "個目で必要プロパティ〝type〟が見つかりませんでした。", "error");
        stop();
        return;
      }
      switch (elem.type) {
        case "direct_jar":
          if (checkMust(elem, ["type", "name", "url"], "配列mods内" + i + "個目で")) return;
          break;
        case "direct_zip":
          if (checkMust(elem, ["type", "url"], "配列mods内" + i + "個目で")) return;
          break;
        default:
          addLog("配列mods内" + i + "個目で、typeプロパティに〝" + elem.type + "〟という値は無効です。", "error");
          stop();
          return;
      }
    }

    for (let i = 0; i < config.otherFiles.length; i++) {
      const elem = config.otherFiles[i];
      if (!Object.keys(elem).includes("url")) {
        addLog("配列mods内" + i + "個目に必要プロパティ〝url〟が見つかりませんでした。", "error");
        stop();
        return;
      }
    }
    pCheckConf.classList.add("succeed");
  }
  //2.Diagnosis
  {
    const result = await diagnosisWith(config.forgeVersion, mcPath);
    if (result <= 1) {
      pDiagnosis.classList.replace("succeed", "failed");
      if (result === 0) {
        addLog("Javaがインストールされていないため失敗しました。インストールしてから再度始めてください。", "error");
        stop();
        const res = await ask("Javaがインストールされていませんでした。ダウンロードURLにアクセスしますか？");
        if (res) {
          openUrl("https://www.java.com/ja/download/ie_manual.jsp");
        }
        return;
      } else {
        addLog(`${config.forgeVersion}がインストールされていないため失敗しました。インストールしてから再度始めてください。`, "error");
        stop();
        const res = await ask("Forgeがインストールされていませんでした。Forgeをインストールしますか？");
        if (res) {
          await invoke("get_forge", {
            forge: config.forgeVersion
          });
        }
        return;
      }
    } else {
      pDiagnosis.classList.add("succeed");
    }
  }
  //3.Mod Install
  {
    pDownload.classList.add("ongoing");
    for (const otherFile of config.otherFiles) {
      if (stopped) return;
      await invoke("dl_direct_zip", {
        url: otherFile.url,
        dir: document.getElementById("dircd").textContent,
        isOther: true
      });
      if (stopped) return;
    }
    for (const mod of config.mods) {
      if (mod.type === "direct_jar") {
        if (stopped) return;
        await invoke("dl_direct_jar", {
          url: mod.url,
          name: mod.name,
          dir: document.getElementById("dircd").textContent
        });
        if (stopped) return;
      } else if (mod.type === "direct_zip") {
        if (stopped) return;
        await invoke("dl_direct_zip", {
          url: mod.url,
          dir: document.getElementById("dircd").textContent,
          isOther: false
        });
        if (stopped) return;
      }
    }
    pDownload.classList.replace("ongoing", "succeed");
    if (noAuto === true) {
      success();
      return;
    }
  }
  //4.Make Profile
  {
    pMakeprof.classList.add("ongoing");
    await invoke("make_profile", {
      name: config.name,
      mcDir: mcPath,
      gameDir: dirPath,
      version: config.forgeVersion
    });
    pMakeprof.classList.replace("ongoing", "succeed");
    success();
  }
}

function stop() {
  invoke("dead_end");
  document.getElementById("startbut").classList.remove("cancel");
  document.getElementById("user-input").classList.remove("disappear");
  document.getElementById("startbut").textContent = "START INSTALLING";
  stopElem("p-chec");
  stopElem("p-diag");
  stopElem("p-down");
  stopElem("p-make");
  for (const elem of document.getElementsByClassName("file-progress")) {
    elem.classList.add("dead");
  }
  for (const elem of document.getElementsByClassName("file-pnum")) {
    elem.textContent = "Downloading was killed";
  }
  stopped = true;
}

function success() {
  document.getElementById("startbut").classList.remove("cancel");
  document.getElementById("user-input").classList.remove("disappear");
  document.getElementById("startbut").textContent = "START INSTALLING";
  message("自動導入は成功しました。", { title: "Auto Mod Installer", kind: "info" });
}

async function diagnosisWith(forgeVer, mcPath) {
  const java = await invoke("diagnosis_java");
  const forge = await invoke("diagnosis_forge", {
    "version": forgeVer,
    "mcPath": mcPath ?? ""
  });
  if (!java) return 0;
  if (!forge) return 1;
  return 2;
}