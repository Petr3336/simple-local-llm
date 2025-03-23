// md-editor-config.ts
import { config } from "md-editor-v3";
import RU from "@vavt/cm-extension/dist/locale/ru";

config({
  editorConfig: {
    languageUserDefined: {
      "ru-RU": RU,
    },
  },
});
