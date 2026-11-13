import { Plugin } from "@opencode/plugin"

/**
 * Local compatibility plugin: expose Ensemble's team_* tools on the native
 * tool list instead of leaving them inside OpenCode's Code Mode catalog.
 *
 * Load this AFTER @hueyexe/opencode-ensemble in the `plugins` array so its
 * transform runs after the ensemble tools are registered. No upstream patch.
 */
export default Plugin.define({
  id: "ensemble-native-compat",
  async setup(ctx) {
    await ctx.tool.transform((editor) => {
      for (const tool of editor.list()) {
        if (!tool.name.startsWith("team_")) continue
        editor.update(tool.id, (item) => {
          item.options = { ...(item.options ?? {}), codemode: false }
        })
      }
    })
  },
})
