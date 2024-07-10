import puppeteer from "puppeteer";
// Or import puppeteer from 'puppeteer-core';

function delay(time) {
  return new Promise(function (resolve) {
    setTimeout(resolve, time);
  });
}

// Launch the browser and open a new blank page
const browser = await puppeteer.launch({ headless: false });
const page = await browser.newPage();

page.on("console", async (msg) => {
  let found = msg.text().match(/Average Frame Time: ([\d\.]+)ms/);
  if (found) {
    console.log(found[1]);
    await browser.close();
  }
});

// Set screen size.
await page.setViewport({ width: 1920, height: 1080 });

// Navigate the page to a URL.
await page.goto("http://127.0.0.1:1334");

await delay(30000);

await browser.close();
