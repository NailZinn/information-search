const requiredNumberOfPages = 100;
const requiredNumberOfWordsPerPage = 1000;

const htmlTagsRegex = /<(.|\n)*?>/gm;
const scriptTagRegex = /<script(.|\n)*?>(.|\n)*?<\/script>/gm;
const styleTagRegex = /<style(.|\n)*?>(.|\n)*?<\/style>/gm;
const redundantSpacesRegex = /\s\s+/gm;
const htmlSpecialCharsRegex = /&.+;/gm;
const hrefRegex = /<a\s+(?:[^>]*?\s+)?href=(["'])(http.+?)(["'])/gm;

const formatUrl = (url: string) => {
  const urlWithoutQuery = url.split("?")[0];
  return urlWithoutQuery.endsWith("/")
    ? urlWithoutQuery.slice(0, urlWithoutQuery.length - 1)
    : urlWithoutQuery;
};

const lookForUrlsInPage = Deno.args.length < requiredNumberOfPages;

const visitedUrls = new Set<string>();

let numberOfSavedPages = 0;

await Deno.mkdir("pages").catch(() =>
  console.log("directory pages already exists")
);

const urls = Deno.args.map(formatUrl);

while (numberOfSavedPages < requiredNumberOfPages && urls.length !== 0) {
  const url = urls.shift()!;

  if (visitedUrls.has(url)) continue;

  console.log(url);
  visitedUrls.add(url);

  if (url.endsWith(".exe") || url.endsWith(".apk")) continue;

  const response = await fetch(url).catch(() => {
    console.log(`${url}: fetch failed`);
    return null;
  });

  if (!response || response.status !== 200) continue;

  const html = await response.text().catch(() => {
    console.log("response stream is not convertable to text");
    return null;
  });

  if (!html) continue;

  const text = html
    .replaceAll(scriptTagRegex, " ")
    .replaceAll(styleTagRegex, " ")
    .replaceAll(htmlTagsRegex, " ")
    .replaceAll(htmlSpecialCharsRegex, " ")
    .replaceAll("\n", " ")
    .replaceAll(redundantSpacesRegex, " ")
    .trim();

  if (text.split(" ").length < requiredNumberOfWordsPerPage) continue;

  numberOfSavedPages++;

  await Deno.writeTextFile(`pages/${numberOfSavedPages.toString()}.txt`, text);

  const indexEntry = `${numberOfSavedPages} - ${url}\n`;

  await Deno.writeTextFile("index.txt", indexEntry, { append: true });

  if (!lookForUrlsInPage) continue;

  const urlsOnPage = html
    .matchAll(hrefRegex)
    .map((x) => formatUrl(x[2]))
    .filter((x) => !visitedUrls.has(x));

  for (const urlOnPage of urlsOnPage) {
    urls.push(urlOnPage);
  }
}
