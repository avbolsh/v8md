
CLI-утилита и MCP-сервер для извлечения метаданных из XML-дампа конфигурации 1С:Предприятие. Быстрый доступ к структуре конфигурации через AI-агенты.

## Возможности

- Извлечение метаданных из XML-дампа конфигурации 1С
- Кэширование результатов для быстрого доступа
- Поддержка всех основных типов метаданных
- Встроенный MCP-сервер (`v8md serve`, JSON-RPC over stdio)
- Интеграция с AI-агентами (opencode, cursor и др.) напрямую или через MCP

## Поддерживаемые типы метаданных

| Тип | Описание |
|-----|----------|
| `Catalog` | Справочники |
| `Document` | Документы |
| `InformationRegister` | Регистры сведений |
| `AccumulationRegister` | Регистры накопления |
| `ChartOfCharacteristicTypes` | Планы видов характеристик |
| `Enum` | Перечисления |
| `Constant` | Константы |
| `DocumentJournal` | Журналы документов |
| `ExchangePlan` | Планы обмена |

## Установка

```bash
cargo build --release
```

Бинарник будет в `target/release/v8md.exe`

## Использование

Все команды работают с **именованным проектом** (`--project`). Каждый проект — своя база 1С, кэши не пересекаются. Список проектов по нескольким базам:

```bash
v8md load-dump C:\Dumps\Учет --project Учет
v8md load-dump C:\Dumps\Розница --project Розница
v8md projects

Учет — УчетТорговли (version 3.1.4.1)
Розница — РозничнаяТорговля (version 2.2.0.0)
```

### 1. Загрузка XML-дампа в кэш проекта

```bash
v8md load-dump <путь_к_дампу> --project <имя>
```

Пример:
```bash
v8md load-dump C:\Projects\MyConfig\dump --project Учет
```

Кэш сохраняется в `~/.v8md/projects/<имя>/`. При каждой загрузке кэш проекта перестраивается заново.

### 2. Получение метаданных объекта

```bash
v8md get <Тип.Имя> --project <имя>
```

Примеры:
```bash
# Справочник
v8md get Catalog.Контрагенты --project Учет

# Документ
v8md get Document.Заказ --project Учет

# Регистр сведений
v8md get InformationRegister.КурсыВалют --project Розница

# Регистр накопления
v8md get AccumulationRegister.ТоварныеЗапасы --project Розница

# Перечисление
v8md get Enum.СостоянияЗаказов --project Учет

# Константа
v8md get Constant.ВалютаУчета --project Учет

# Журнал документов
v8md get DocumentJournal.ФинансовыеДокументы --project Учет

# План видов характеристик
v8md get ChartOfCharacteristicTypes.ВидыХарактеристик --project Учет

# План обмена
v8md get ExchangePlan.Мобильные --project Розница
```

#### Опции

```bash
# Только атрибуты (реквизиты)
v8md get Document.Заказ -a --project Учет

# Только табличные части
v8md get Document.Заказ -t --project Учет
```

### 3. Получение типа атрибута

```bash
v8md types <Тип.Объект.Атрибут> --project <имя>
```

Пример:
```bash
v8md types Document.Заказ.Покупатель --project Учет
```

### 4. Список объектов проекта

```bash
# Все объекты
v8md list --project Учет

# Фильтр по типу
v8md list Document --project Учет
v8md list Catalog --project Розница
```

### 5. Список загруженных проектов

```bash
v8md projects
```

### 6. Запуск в режиме MCP-сервера

```bash
v8md serve
```

Открывает JSON-RPC over stdio с инструментами: `v8md_load_dump`, `v8md_projects`, `v8md_get`, `v8md_types`, `v8md_list`. Для всех инструментов (кроме `v8md_projects`) параметр `project` обязателен.

Пример настройки для клиентов, поддерживающих MCP:

```json
{
  "mcpServers": {
    "v8md": {
      "command": "C:\\path\\to\\v8md.exe",
      "args": ["serve"]
    }
  }
}
```

## Формат вывода

### Справочник

```json
{
  "object": "Catalog.Контрагенты",
  "type": "Catalog",
  "synonym": "Контрагенты",
  "properties": {
    "Код": { "type": "String", "length": 9, "allowed_length": "Variable" },
    "Наименование": { "type": "String", "length": 30, "allowed_length": "Variable" },
    "ЭлектроннаяПочта": { "type": "String", "length": 40, "allowed_length": "Variable" },
    "ВидЦен": { "type": "CatalogRef", "ref": "ВидыЦен" }
  }
}
```

### Документ

```json
{
  "object": "Document.Заказ",
  "type": "Document",
  "synonym": "Заказ",
  "properties": {
    "Номер": { "type": "String", "length": 9, "allowed_length": "Variable" },
    "Дата": { "type": "Date", "date_fractions": "DateTime" },
    "Покупатель": { "type": "CatalogRef", "ref": "Контрагенты" },
    "Сумма": { "type": "Number", "precision": 10, "scale": 2, "allowed_sign": "Any" }
  },
  "tabular_sections": {
    "Товары": {
      "columns": {
        "Товар": { "type": "CatalogRef", "ref": "Товары" },
        "Количество": { "type": "Number", "precision": 10, "scale": 0 },
        "Цена": { "type": "Number", "precision": 10, "scale": 2 }
      }
    }
  }
}
```

### Регистр сведений

```json
{
  "object": "InformationRegister.КурсыВалют",
  "type": "InformationRegister",
  "synonym": "Курсы валют",
  "periodicity": "Day",
  "write_mode": "Independent",
  "main_filter_on_period": true,
  "dimensions": {
    "Валюта": { "type": "CatalogRef", "ref": "Валюты" }
  },
  "resources": {
    "Курс": { "type": "Number", "precision": 10, "scale": 2, "allowed_sign": "Nonnegative" }
  }
}
```

## Интеграция с AI-агентами

### opencode

1. Скопируйте бинарник в `.opencode/bin/v8md.exe`

2. Создайте плагин `.opencode/plugins/v8md.ts`:

```typescript
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
```

3. Добавьте плагин в конфигурацию opencode.

### Примеры промптов для AI-агента

```
Покажи структуру справочника Контрагенты
```

```
Какие реквизиты у документа Заказ?
```

```
Какой тип у реквизита Покупатель в документе Заказ?
```

```
Покажи измерения и ресурсы регистра сведений КурсыВалют
```

## Архитектура

```
v8md/
├── src/
│   ├── main.rs           # CLI (clap)
│   ├── model.rs          # Структуры данных (MdObject и т.д.)
│   ├── parser.rs         # Парсинг XML метаданных
│   ├── type_parser.rs    # Парсинг типов данных
│   ├── cache.rs          # Кэширование (~/.v8md/)
│   ├── commands.rs       # Команды CLI
│   └── mcp.rs            # MCP-сервер (JSON-RPC over stdio)
├── .opencode/
│   ├── bin/v8md.exe      # Бинарник
│   └── plugins/v8md.ts   # Плагин для opencode
├── Cargo.toml
└── README.md
```

## Лицензия

MIT
