/**
 * @jerekode/rtk — default export is the OpenCode2/Bun plugin module.
 */

export { name, hooks, default } from "./opencode2.ts";
export {
  alreadyRtk,
  rewriteWithTable,
  rewriteCommand,
  applyToolExecuteBefore,
} from "./rewrite.ts";
