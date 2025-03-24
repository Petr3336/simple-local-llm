# Student LLM Tauri Project

Этот проект – студенческий проект для запуска LLM (Large Language Model) с использованием Tauri для создания приложения, Vue.js для фронтенда и Vuetify для оформления интерфейса.

## Особенности проекта

- **Потоковый вывод:** Отображение данных сгенерированных LLM в реальном времени.
- **Кроссплатформенность:** Построено с использованием Tauri поддерживающем Windows, Linux, Android.

## Требования

- **Node.js** (версия ≥ 14.x)
- **npm** или **yarn**
- **Rust** (стабильная версия)

## Установка

### 1. Клонирование репозитория

```bash
git clone https://github.com/Petr3336/simple-local-llm.git
cd your-repo
```

### 2. Установка зависимостей Node.js

Используйте npm в корне проекта:

```bash
npm install
```

### 3. Установка зависимостей Rust

Установите компилятор языка Rust

## Запуск проекта

### Режим разработки

Для одновременного запуска фронтенда и бэкенда Tauri выполните следующую команду в корне проекта:

```bash
npx tauri dev
```

Данная команда автоматически загрузит зависимости Rust для сборки

Используйте `--features llama_cpp` для сборки с поддержкой [llama.cpp](https://github.com/ggml-org/llama.cpp)

```bash
npx tauri dev --features llama_cpp
```

### Сборка проекта для продакшена

Чтобы собрать установочный пакет десктопного приложения, выполните следующую команду в корне проекта:

```bash
npx tauri build
```

Используйте `--features llama_cpp` для сборки с поддержкой [llama.cpp](https://github.com/ggml-org/llama.cpp)

```bash
npx tauri build --features llama_cpp
```

- Исполняемый файл приложения будет расположен по пути `src-tauri\target\release`
- Установочный пакет приложения будет расположен по пути `src-tauri\target\release\bundle`

## Используемые библиотеки и лицензии

При разработке проекта используются следующие библиотеки:

- **Tauri**
  - **Лицензия:** MPL-2.0 / MIT / Apache-2.0 (двойное лицензирование)
  - **Репозиторий:** [Tauri GitHub](https://github.com/tauri-apps/tauri)

- **Vue.js**
  - **Лицензия:** MIT
  - **Сайт:** [Vue.js Official Website](https://vuejs.org/)

- **Vuetify**
  - **Лицензия:** MIT
  - **Сайт:** [Vuetify Official Website](https://vuetifyjs.com/)

- **Reqwest** (Rust HTTP клиент)
  - **Лицензия:** MIT / Apache-2.0 (двойное лицензирование)
  - **Репозиторий:** [Reqwest GitHub](https://github.com/seanmonstar/reqwest)

- **Serde и serde_json** (для работы с JSON)
  - **Лицензия:** MIT / Apache-2.0 (двойное лицензирование)
  - **Репозиторий:** [Serde GitHub](https://github.com/serde-rs/serde)

- **Futures-util**
  - **Лицензия:** MIT / Apache-2.0 (двойное лицензирование)
  - **Репозиторий:** [Futures GitHub](https://github.com/rust-lang/futures-rs)

## Дополнительная информация

- Проект создан в образовательных целях и не предназначен для коммерческого использования.

## Third-Party Libraries & Licenses

This project makes use of the following third-party libraries:

- [Vue.js](https://vuejs.org) — MIT License
  Copyright (c) Evan You

- [Vuetify](https://vuetifyjs.com) — MIT License

- [Tauri](https://tauri.app) — MPL-2.0 OR MIT OR Apache-2.0

- [Reqwest](https://github.com/seanmonstar/reqwest) — Apache-2.0 / MIT


