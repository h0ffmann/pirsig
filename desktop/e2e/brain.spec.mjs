// The whole brain through the UI, on a fresh brain: PARA, notes and links,
// Hippos, feeds, Plasticity and readiness. One story, in order, because each
// step builds on what the last one filed.
import { expect, test } from "@playwright/test";

test.describe.configure({ mode: "serial" });

let page;
const errors = [];

test.beforeAll(async ({ browser }) => {
  page = await browser.newPage();
  page.on("pageerror", (e) => errors.push(e.message));
  // an API refusal is logged as a failed load; the UI shows those itself
  page.on("console", (m) => m.type() === "error" && !m.text().startsWith("Failed to load resource") && errors.push(m.text()));
  await page.goto("/");
});

test.afterAll(async () => {
  await page.close();
});

test.afterEach(() => {
  expect(errors, "no errors in the page").toEqual([]);
});

const nav = (name) => page.locator("#nav").getByRole("link", { name: new RegExp(`^${name}\\b`) }).click();
const dialog = () => page.getByRole("dialog");
// a <select>'s name ends with its chosen option: match the start
const choose = (label) => dialog().getByRole("combobox", { name: new RegExp(`^${label}`) });
const row = (text) => page.locator(".list > li").filter({ hasText: text });
const toast = (text) => expect(page.locator("#toasts")).toContainText(text);

// Capture a fleeting note from the sidebar; returns its zid.
async function capture(title) {
  await page.getByLabel("Capture a fleeting note").fill(title);
  await page.getByLabel("Capture a fleeting note").press("Enter");
  const t = page.locator(".toast", { hasText: `Captured “${title}” in the inbox as` });
  await expect(t).toBeVisible();
  return (await t.textContent()).match(/as (\S+)$/)[1];
}

let first;
let second;

test("today and quick capture", async () => {
  await expect(page.getByRole("heading", { name: "Today" })).toBeVisible();
  first = await capture("Waves break when steep");
  second = await capture("Rogue waves are rare");
  await expect(page.locator("#nav").getByRole("link", { name: /^Slip-box/ })).toContainText("2");
  await nav("Slip-box");
  await expect(row("Waves break when steep")).toBeVisible();
});

test("areas: create, edit, archive", async () => {
  await nav("Areas");
  await page.getByRole("button", { name: "New area" }).click();
  await dialog().getByLabel("Name").fill("Health");
  await dialog().getByLabel("Description").fill("Run, sleep, eat well");
  await dialog().getByRole("button", { name: "Save" }).click();
  await toast("Area Health created");
  await row("Health").getByRole("button", { name: "Edit" }).click();
  await dialog().getByLabel("Name").fill("Health and body");
  await dialog().getByRole("button", { name: "Save" }).click();
  await expect(row("Health and body")).toBeVisible();

  await page.getByRole("button", { name: "New area" }).click();
  await dialog().getByLabel("Name").fill("Old hobbies");
  await dialog().getByRole("button", { name: "Save" }).click();
  await row("Old hobbies").getByRole("button", { name: "Archive" }).click();
  await toast("Archived");
  await expect(row("Old hobbies")).toHaveCount(0);
  await page.getByRole("link", { name: "Archive" }).click();
  await row("Old hobbies").getByRole("button", { name: "Unarchive" }).click();
  await toast("Back from the archive");
});

test("projects and tasks", async () => {
  await nav("Projects");
  await page.getByRole("button", { name: "New project" }).click();
  await dialog().getByLabel("Name").fill("Run a 10k");
  await choose("Area").selectOption({ label: "Health and body" });
  await dialog().getByLabel("Due on").fill("2026-12-01");
  await dialog().getByRole("button", { name: "Save" }).click();
  await toast("Project Run a 10k created");
  await page.getByRole("link", { name: "Run a 10k" }).click();
  await expect(page.getByRole("heading", { name: "Run a 10k" })).toBeVisible();

  const add = page.getByPlaceholder("Add a task and press Enter");
  await add.fill("Buy shoes");
  await add.press("Enter");
  await expect(row("Buy shoes")).toBeVisible();
  await page.getByPlaceholder("Add a task and press Enter").fill("Train three times a week");
  await page.getByPlaceholder("Add a task and press Enter").press("Enter");
  await expect(row("Train three times a week")).toBeVisible();

  await page.getByLabel("Done: Buy shoes").check();
  await expect(row("Buy shoes")).toHaveClass(/done/);
  await expect(page.getByRole("heading", { name: "Tasks (1 open)" })).toBeVisible();

  await row("Train three times a week").getByRole("button", { name: "Edit" }).click();
  await dialog().getByLabel("Task").fill("Train four times a week");
  await dialog().getByLabel("Due on").fill("2026-11-15");
  await dialog().getByRole("button", { name: "Save" }).click();
  await toast("Task saved");
  await expect(row("Train four times a week")).toContainText("due");

  await row("Train four times a week").getByRole("button", { name: "Delete" }).click();
  await dialog().getByRole("button", { name: "Delete" }).click();
  await toast("Task deleted");
  await expect(row("Train four times a week")).toHaveCount(0);

  await nav("Tasks");
  await page.getByRole("link", { name: "Done" }).click();
  await expect(row("Buy shoes")).toBeVisible();
});

test("notes: write, link, unlink, pull into a project", async () => {
  await nav("Notes");
  await page.getByRole("link", { name: "New note" }).click();
  await page.getByLabel("Title").fill("Steep waves break");
  await page.getByLabel("Kind").selectOption("permanent");
  await page.getByLabel("Body (Markdown)").fill(`Waves break once their **steepness** passes a limit.\n\nSee [[${first}]]. #ocean`);
  await expect(page.locator(".editor .note-body strong")).toHaveText("steepness");
  await page.getByRole("button", { name: "File the note" }).click();
  await expect(page.getByRole("heading", { name: "Steep waves break" })).toBeVisible();
  await expect(page.locator(".note-body a.wikilink")).toHaveAttribute("href", `#/notes/${first}`);
  await expect(page.locator(".sub")).toContainText("#ocean");
  const zid = decodeURIComponent(page.url().split("#/notes/")[1]);

  await page.getByRole("button", { name: "Link…" }).click();
  await dialog().getByLabel("Note to link to").fill(second);
  await dialog().getByLabel("Why they belong together").fill("rogue waves are the steepest");
  await dialog().getByRole("button", { name: "Link" }).click();
  await toast("Linked");
  await expect(row("Rogue waves are rare")).toContainText("rogue waves are the steepest");

  await page.locator(".note-body a.wikilink").click();
  await expect(page.getByRole("heading", { name: "Waves break when steep" })).toBeVisible();
  await expect(row("Steep waves break")).toBeVisible();
  await page.goBack();

  await row("Rogue waves are rare").getByRole("button", { name: "Unlink" }).click();
  await dialog().getByRole("button", { name: "Unlink" }).click();
  await toast("Unlinked");
  await expect(row("Rogue waves are rare")).toHaveCount(0);

  await page.getByRole("link", { name: "Edit" }).click();
  await page.getByLabel("Title").fill("Steep waves break, always");
  await page.getByLabel("Body (Markdown)").press("Meta+s");
  await expect(page.getByRole("heading", { name: "Steep waves break, always" })).toBeVisible();

  await page.getByRole("button", { name: "Pull into project" }).click();
  await choose("Project").selectOption({ label: "Run a 10k" });
  await dialog().getByLabel("Why the project needs it").fill("pacing like a wave");
  await dialog().getByRole("button", { name: "Pull in" }).click();
  await toast("Pulled into the project");

  await page.getByRole("button", { name: "Markdown" }).click();
  await expect(dialog().locator("pre")).toContainText(`zid: "${zid}"`);
  await dialog().getByRole("button", { name: "Cancel" }).click();

  await nav("Projects");
  await page.getByRole("link", { name: "Run a 10k" }).click();
  await expect(row("Steep waves break, always")).toContainText("pacing like a wave");
  await row("Steep waves break, always").getByRole("button", { name: "Drop" }).click();
  await toast("Dropped from the project");

  await page.goto(`/#/notes?tag=ocean`);
  await expect(row("Steep waves break, always")).toBeVisible();
  await page.goto(`/#/notes?q=Rogue`);
  await expect(row("Rogue waves are rare")).toBeVisible();
  await row("Rogue waves are rare").getByRole("link", { name: "Rogue waves are rare" }).click();
  await page.getByRole("button", { name: "Delete" }).click();
  await dialog().getByRole("button", { name: "Delete" }).click();
  await toast("Note deleted");
  await expect(page.getByRole("heading", { name: "Notes" })).toBeVisible();
});

test("resources and literature notes", async () => {
  await nav("Resources");
  await page.getByRole("button", { name: "New resource" }).click();
  await dialog().getByLabel("Title").fill("WAVEWATCH III manual");
  await dialog().getByLabel("URL").fill("https://example.org/ww3");
  await choose("Kind").selectOption("book");
  await choose("Area").selectOption({ label: "Health and body" });
  await dialog().getByRole("button", { name: "Save" }).click();
  await toast("Resource added");
  await page.getByRole("link", { name: "WAVEWATCH III manual" }).click();
  await page.getByRole("link", { name: "Write a literature note" }).click();
  await expect(page.getByLabel("Kind")).toHaveValue("literature");
  await page.getByLabel("Title").fill("WW3 solves the action balance");
  await page.getByLabel("Body (Markdown)").fill("The model evolves the wave action spectrum in time.");
  await page.getByRole("button", { name: "File the note" }).click();
  await expect(page.getByRole("link", { name: "WAVEWATCH III manual" })).toBeVisible();

  await page.getByRole("link", { name: "WAVEWATCH III manual" }).click();
  await page.getByRole("button", { name: "Archive" }).click();
  await expect(page.getByRole("button", { name: "Unarchive" })).toBeVisible();
});

test("a literature note without a source is refused, with the reason", async () => {
  await page.goto("/#/notes/new?kind=literature");
  await page.getByLabel("Title").fill("Unsourced claim");
  await page.getByRole("button", { name: "File the note" }).click();
  await expect(page.getByRole("alert")).toContainText(/source/i);
});

test("hippos: create, grow, weave, add a fixation", async () => {
  await nav("Hippos");
  await page.getByRole("button", { name: "New Hippo" }).click();
  await dialog().getByLabel("Name").fill("Sea mind");
  await dialog().getByLabel("Fixations").fill("waves");
  await dialog().getByRole("button", { name: "Save" }).click();
  await toast("Hippo Sea mind created");
  await page.getByRole("link", { name: "Sea mind" }).click();
  await page.getByRole("link", { name: "Grow a note" }).click();
  await page.getByLabel("Title").fill("Waves again");
  await page.getByLabel("Body (Markdown)").fill("Everything is waves today.");
  await page.getByRole("button", { name: "File the note" }).click();
  await expect(page.getByRole("link", { name: "Sea mind" })).toBeVisible();
  await page.getByRole("link", { name: "Sea mind" }).click();
  await expect(page.getByRole("heading", { name: "waves", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Weave" }).click();
  await toast("Woven");

  await page.getByRole("button", { name: "Edit" }).click();
  await dialog().getByLabel("Fixations").fill("waves, tides");
  await dialog().getByRole("button", { name: "Save" }).click();
  await toast("Hippo saved");
  await expect(page.getByRole("heading", { name: "tides", exact: true })).toBeVisible();
});

test("feeds: subscribe and unsubscribe", async () => {
  await nav("Feeds and arXiv");
  await page.getByRole("button", { name: "Subscribe" }).click();
  await dialog().getByLabel("RSS or Atom URL").fill("https://example.org/feed.xml");
  await dialog().getByRole("button", { name: "Subscribe" }).click();
  await toast("Subscribed");
  await expect(row("https://example.org/feed.xml")).toContainText("never pulled");
  await row("https://example.org/feed.xml").getByRole("button", { name: "Unsubscribe" }).click();
  await dialog().getByRole("button", { name: "Unsubscribe" }).click();
  await toast("Unsubscribed");
});

test("plasticity: practise one exercise", async () => {
  await nav("Plasticity");
  await page.getByRole("button", { name: "Practise" }).click();
  await expect(page.locator(".prompt")).not.toBeEmpty();
  const choice = page.locator("button.choice").first();
  if (await choice.count()) await choice.click();
  else {
    await page.getByLabel("Your answer").fill("42");
    await page.getByRole("button", { name: "Answer" }).click();
  }
  await expect(page.locator(".verdict")).toContainText(/Right\.|Not quite/);
  await expect(page.getByRole("button", { name: "Next exercise" })).toBeVisible();
});

test("readiness and the slip-box views", async () => {
  await nav("Readiness");
  await expect(page.locator(".sub")).toContainText("checks ready");
  await nav("Slip-box");
  await page.getByRole("link", { name: "Tree" }).click();
  await expect(page.locator(".tree")).toContainText("Waves break when steep");
  await page.getByRole("link", { name: "Tags" }).click();
  await expect(page.locator(".tagcloud")).toContainText("#ocean");
  await page.getByRole("link", { name: "Random note" }).click();
  await expect(page).toHaveURL(/#\/notes\//);
});

test("deleting an area with projects asks twice, then takes them along", async () => {
  await nav("Areas");
  await row("Health and body").getByRole("button", { name: "Delete" }).click();
  await dialog().getByRole("button", { name: "Delete" }).click();
  await expect(dialog()).toContainText("force");
  await dialog().getByRole("button", { name: "Delete all" }).click();
  await toast("Deleted area Health and body");
  await expect(row("Health and body")).toHaveCount(0);
  await nav("Projects");
  await expect(row("Run a 10k")).toHaveCount(0);
});
