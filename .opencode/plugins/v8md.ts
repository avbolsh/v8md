import { Plugin, tool } from "@opencode-ai/plugin"
import { $ } from "bun"
import path from "path"
import { z } from "zod"

export const V8MDPlugin: Plugin = async () => {
  return {
    tool: {
      "v8md-load": tool({
        description: "Загрузить XML-дамп конфигурации 1С в кэш. Использовать один раз для загрузки метаданных.",
        args: z.object({
          path: z.string().describe("Путь к папке с Configuration.xml"),
        }),
        async execute({ path: dumpPath }, { directory }) {
          const v8mdExe = path.join(directory, "bin", "v8md.exe")
          const result = await $`${v8mdExe} load-dump ${dumpPath}`.text()
          return result
        },
      }),

      "v8md-get": tool({
        description: "Получить метаданные объекта 1С. Формат: Тип.Имя (например: Document.Заказ, Catalog.Контрагенты).",
        args: z.object({
          object: z.string().describe("Имя объекта, например: Catalog.Контрагенты, Document.Заказ"),
          attributes_only: z.boolean().optional().describe("Показать только атрибуты"),
          tabular_only: z.boolean().optional().describe("Показать только табличные части"),
        }),
        async execute({ object, attributes_only, tabular_only }, { directory }) {
          const v8mdExe = path.join(directory, "bin", "v8md.exe")
          const args = ["get", object]
          if (attributes_only) args.push("-a")
          if (tabular_only) args.push("-t")
          const result = await $`${v8mdExe} ${args}`.json()
          return JSON.stringify(result, null, 2)
        },
      }),

      "v8md-types": tool({
        description: "Получить типы атрибута объекта 1С. Формат: Тип.Объект.Атрибут (например: Document.Заказ.Покупатель).",
        args: z.object({
          path: z.string().describe("Путь к атрибуту, например: Document.Заказ.Покупатель"),
        }),
        async execute({ path: attrPath }, { directory }) {
          const v8mdExe = path.join(directory, "bin", "v8md.exe")
          const result = await $`${v8mdExe} types ${attrPath}`.json()
          return JSON.stringify(result, null, 2)
        },
      }),

      "v8md-list": tool({
        description: "Список всех объектов 1С в кэше. Можно указать фильтр по типу.",
        args: z.object({
          kind: z.string().optional().describe("Фильтр по типу: Catalog, Document, InformationRegister и т.д."),
        }),
        async execute({ kind }, { directory }) {
          const v8mdExe = path.join(directory, "bin", "v8md.exe")
          const args = ["list"]
          if (kind) args.push(kind)
          const result = await $`${v8mdExe} ${args}`.text()
          return result
        },
      }),
    },
  }
}
