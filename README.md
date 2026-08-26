
CLI-утилита и MCP-сервер для извлечения метаданных из XML-дампа конфигурации 1С:Предприятие. Быстрый доступ к структуре конфигурации через AI-агенты.

## Возможности

- Извлечение метаданных из XML-дампа конфигурации 1С
- Кэширование результатов для быстрого доступа
- Поддержка всех основных типов метаданных
- Парсинг программных интерфейсов (BSL-модулей), включая модули менеджеров
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
| `CommonModule` | Общие модули |

## Установка

```bash
cargo build --release
```

Бинарник будет в `target/release/v8md.exe`

## Использование

### 1. Загрузка XML-дампа в кэш

```bash
v8md load-dump <путь_к_дампу>
```

Пример:
```bash
v8md load-dump C:\Projects\MyConfig\dump
```

Кэш сохраняется в `~/.v8md/`. При каждой загрузке кэш перестраивается заново.

Вместе с метаданными парсятся BSL-модули: модуль менеджера объекта (`Ext/ManagerModule.bsl`) и общие модули (`CommonModules/<Имя>/Ext/Module.bsl`) с описаниями методов из комментариев.

### 2. Получение метаданных объекта

```bash
v8md get <Тип.Имя>
```

Примеры:
```bash
# Справочник
v8md get Catalog.Контрагенты

# Документ
v8md get Document.Заказ

# Регистр сведений
v8md get InformationRegister.КурсыВалют

# Регистр накопления
v8md get AccumulationRegister.ТоварныеЗапасы

# Перечисление
v8md get Enum.СостоянияЗаказов

# Константа
v8md get Constant.ВалютаУчета

# Журнал документов
v8md get DocumentJournal.ФинансовыеДокументы

# План видов характеристик
v8md get ChartOfCharacteristicTypes.ВидыХарактеристик

# План обмена
v8md get ExchangePlan.Мобильные

# Общий модуль
v8md get CommonModule.РаботаСПочтой
```

#### Опции

```bash
# Только атрибуты (реквизиты)
v8md get Document.Заказ -a

# Только табличные части
v8md get Document.Заказ -t
```

### 3. Получение типа атрибута

```bash
v8md types <Тип.Объект.Атрибут>
```

Пример:
```bash
v8md types Document.Заказ.Покупатель
```

### 4. Список объектов

```bash
# Все объекты
v8md list

# Фильтр по типу
v8md list CommonModule
v8md list Catalog
v8md list Document
```

### 5. Запуск в режиме MCP-сервера

```bash
v8md serve
```

Открывает JSON-RPC over stdio с инструментами: `v8md_load_dump`, `v8md_get`, `v8md_types`, `v8md_list`.

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

### Общий модуль

```json
{
  "object": "CommonModule.РаботаСПочтойВызовСервера",
  "type": "CommonModule",
  "context": {
    "server": true,
    "server_call": true
  },
  "methods": [
    {
      "name": "ПолучитьПисьма",
      "kind": "Function",
      "export": true,
      "params": [
        { "name": "Количество" },
        { "name": "ИспользоватьIMAP" },
        { "name": "ПочтовыйЯщикIMAP" }
      ]
    },
    {
      "name": "ПроверитьЭлектронныйАдрес",
      "kind": "Function",
      "export": true,
      "params": [
        { "name": "ЭлектроннаяПочта", "type": "Строка", "description": "адрес электронной почты" }
      ],
      "returns": "Истина, если адрес корректный"
    }
  ]
}
```

#### Контекст модуля

| Поле | Описание |
|------|----------|
| `server` | Серверный модуль |
| `server_call` | Можно вызвать с клиента (серверная функция) |
| `client` | Клиент управляемого приложения |
| `global` | Глобальный модуль |
| `privileged` | Привилегированный режим |

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
        description: "Получить метаданные объекта 1С. Формат: Тип.Имя (например: Document.Заказ, CommonModule.РаботаСПочтой).",
        args: z.object({
          object: z.string().describe("Имя объекта, например: Catalog.Контрагенты, Document.Заказ, CommonModule.РаботаСПочтой"),
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
          kind: z.string().optional().describe("Фильтр по типу: Catalog, Document, CommonModule и т.д."),
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
Покажи методы модуля РаботаСПочтойВызовСервера
```

```
Какой тип у реквизита Покупатель в документе Заказ?
```

```
Покажи измерения и ресурсы регистра сведений КурсыВалют
```

```
Напиши функцию на 1С, которая использует CommonModule.РаботаСПочтой для отправки письма
```

## Архитектура

```
v8md/
├── src/
│   ├── main.rs           # CLI (clap)
│   ├── model.rs          # Структуры данных (MdObject, BsModule и т.д.)
│   ├── parser.rs         # Парсинг XML метаданных
│   ├── module_parser.rs  # Парсинг BSL модулей
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
