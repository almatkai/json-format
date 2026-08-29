/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {
  /** Auto Paste - Paste the formatted JSON to an active app */
  "autopaste": boolean,
  /** Indentation - Output file indentation */
  "indent": "tab" | "2" | "4" | "8",
  /** Max Initial Lines - Maximum number of lines to display initially (for large JSON files) */
  "maxInitialLines": string
}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `formatClipboard` command */
  export type FormatClipboard = ExtensionPreferences & {}
  /** Preferences accessible in the `formatSelection` command */
  export type FormatSelection = ExtensionPreferences & {}
  /** Preferences accessible in the `formatSelectionAndShow` command */
  export type FormatSelectionAndShow = ExtensionPreferences & {}
  /** Preferences accessible in the `index` command */
  export type Index = ExtensionPreferences & {}
  /** Preferences accessible in the `formatJsonLines` command */
  export type FormatJsonLines = ExtensionPreferences & {}
  /** Preferences accessible in the `formatToJsonValue` command */
  export type FormatToJsonValue = ExtensionPreferences & {}
}

declare namespace Arguments {
  /** Arguments passed to the `formatClipboard` command */
  export type FormatClipboard = {}
  /** Arguments passed to the `formatSelection` command */
  export type FormatSelection = {}
  /** Arguments passed to the `formatSelectionAndShow` command */
  export type FormatSelectionAndShow = {}
  /** Arguments passed to the `index` command */
  export type Index = {}
  /** Arguments passed to the `formatJsonLines` command */
  export type FormatJsonLines = {}
  /** Arguments passed to the `formatToJsonValue` command */
  export type FormatToJsonValue = {}
}

