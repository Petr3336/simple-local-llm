let providerActivator;
let providerMeta = [];

describe('Model Chat Form (провайдеры)', () => {
  it('1. Получает список провайдеров и их названия', async () => {
    const providerInput = await $('#model-provider');
    providerActivator = await providerInput.parentElement();
    await providerActivator.waitForExist();
    await providerActivator.click();
  
    // Ждём появления хотя бы одного элемента
    await browser.waitUntil(async () => {
      const items = await $$('div[role="listbox"] .v-list-item');
      return items.length > 0;
    }, {
      timeout: 3000,
      timeoutMsg: '❌ Провайдеры не найдены',
    });
  
    // Используем browser.execute для чтения DOM напрямую
    const names = await browser.execute(() => {
      return Array.from(document.querySelectorAll('div[role="listbox"] .v-list-item')).map(el => {
        const title = el.querySelector('.v-list-item-title');
        return title?.innerText?.trim() || el.innerText?.trim() || '(без названия)';
      });
    });
  
    providerMeta = names.map((name, index) => ({
      index,
      name,
    }));
  
    await providerActivator.click(); // Закрываем список
    console.log('🔍 Найдено провайдеров:', providerMeta.map(p => p.name).join(', '));
  });
  

  after(() => {
    providerMeta.forEach(({ index, name }) => {
      describe(`🔁 Провайдер "${name}"`, () => {
        it('2. Выбирает провайдера', async () => {
          await selectVuetifyItemByIndex(providerActivator, index);
          await browser.pause(1000);
        });

        it('3. Выбирает первую модель', async () => {
          const modelInput = await $('#model-name');
          const modelActivator = await modelInput.parentElement();
          await modelActivator.click();
          await selectVuetifyItemByIndex(modelActivator, 0);
        });

        it('4. Вводит prompt', async () => {
          const promptField = await $('#model-prompt');
          await promptField.click();
          await browser.keys(['Control', 'a']);
          await browser.keys('Backspace');
          await promptField.addValue('Write "Hello"');
        });

        it('5. Генерирует ответ и проверяет результат', async () => {
          const generateBtn = await $('#generate');
          await generateBtn.click();

          const preview = await $('.md-preview');
          await preview.waitForDisplayed({ timeout: 15000 });

          const outputText = await preview.getText();
          console.log(`✅ Ответ модели "${name}": ${outputText}`);
          expect(outputText).toContain('Hello');
        });
      });
    });
  });
});



async function selectVuetifyItemByIndex(activatorElement, index) {
  await activatorElement.waitForClickable({ timeout: 3000 });

  // Клик по селекту
  await activatorElement.click();
  await browser.pause(500);

  // Ждём, пока появится список
  await browser.waitUntil(async () => {
    const items = await $$('div[role="listbox"] .v-list-item');
    return items.length > index;
  }, {
    timeout: 3000,
    timeoutMsg: `❌ Не удалось найти элемент с индексом ${index} в списке`,
  });

  const items = await $$('div[role="listbox"] .v-list-item');
  await items[index].scrollIntoView();
  await items[index].click();
}