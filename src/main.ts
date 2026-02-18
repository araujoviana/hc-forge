import { createApp } from "vue";
import App from "./App.vue";
import "./tailwind.css";

function shouldDisableAutoCap(input: HTMLInputElement): boolean {
  const type = (input.getAttribute("type") ?? "text").toLowerCase();
  return !["checkbox", "radio", "range", "file", "button", "submit", "reset", "color"].includes(type);
}

function hardenTextEntryInput(node: Element) {
  if (node instanceof HTMLInputElement && shouldDisableAutoCap(node)) {
    node.setAttribute("autocapitalize", "off");
    node.setAttribute("autocorrect", "off");
    node.setAttribute("spellcheck", "false");
    return;
  }

  if (node instanceof HTMLTextAreaElement) {
    node.setAttribute("autocapitalize", "off");
    node.setAttribute("autocorrect", "off");
    node.setAttribute("spellcheck", "false");
  }
}

function applyInputTypingGuards(root: ParentNode = document) {
  root.querySelectorAll("input,textarea").forEach(hardenTextEntryInput);
}

applyInputTypingGuards();

const observer = new MutationObserver((mutations) => {
  for (const mutation of mutations) {
    for (const added of mutation.addedNodes) {
      if (!(added instanceof Element)) {
        continue;
      }
      hardenTextEntryInput(added);
      applyInputTypingGuards(added);
    }
  }
});

observer.observe(document.documentElement, { childList: true, subtree: true });

createApp(App).mount("#app");
