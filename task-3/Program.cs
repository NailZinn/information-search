var index = (await File.ReadAllTextAsync("../artifacts/index.txt"))
    .Trim()
    .Split("\n");

if (!File.Exists("../artifacts/inverted-index.txt"))
{
    await CreateInvertedIndex();
}

var invertedIndex = (await File.ReadAllTextAsync("../artifacts/inverted-index.txt"))
    .Trim()
    .Split("\n")
    .Select(x => x.Split(" - "))
    .ToDictionary(x => x[0], x => x[1].Split(", "))
    .GetAlternateLookup<ReadOnlySpan<char>>();

if (args.Length == 0)
{
    Console.WriteLine("No expression provided");
}

var allDocuments = index
    .Select(x => x.Split(" - "))
    .ToDictionary(x => x[0], x => x[1]);
var allDocumentNames = allDocuments.Keys;

var expression = args[0].Split(" ");
var tokenDocuments = new List<IEnumerable<string>>();
var operators = new List<string>();

for (var i = 0; i < expression.Length; i++)
{
    if (expression[i] is "&" or "|")
    {
        operators.Add(expression[i]);
        continue;
    }

    tokenDocuments.Add(GetTokenDocuments(expression[i]));
}

var result = Enumerable.Empty<string>();
var andResult = Enumerable.Empty<string>();

if (operators.Count != 0 && operators[0] == "&")
{
    andResult = tokenDocuments[0];
}
else
{
    result = tokenDocuments[0];
}

for (var i = 0; i < operators.Count; i++)
{
    if (operators[i] == "|" && i - 1 > 0 && operators[i - 1] == "&")
    {
        result = result.Union(andResult);
        andResult = [];
    }
    if (operators[i] == "|" && i + 1 < operators.Count && operators[i + 1] == "&")
    {
        andResult = tokenDocuments[i + 1];
        continue;
    }
    if (operators[i] == "|" && (i + 1 >= operators.Count || operators[i + 1] != "&"))
    {
        result = Process(result, "|", tokenDocuments[i + 1]);
        continue;
    }
    if (operators[i] == "&")
    {
        andResult = Process(andResult, "&", tokenDocuments[i + 1]);
        continue;
    }
}

result = result.Union(andResult);

Console.WriteLine(string.Join("\n", result.OrderBy(int.Parse).Select(x => $"{x} - {allDocuments[x]}")));

IEnumerable<string> GetTokenDocuments(string token)
{
    var span = token.AsSpan();
    var word = span[1..];

    return (span[0] == '!') switch
    {
        true => invertedIndex.ContainsKey(word) ? allDocumentNames.Except(invertedIndex[word]) : allDocumentNames,
        false => invertedIndex.ContainsKey(span) ? invertedIndex[span] : []
    };
}

IEnumerable<string> Process(IEnumerable<string> currentDocuments, string @operator, IEnumerable<string> documents)
{
    return @operator switch
    {
        "&" => currentDocuments.Intersect(documents),
        "|" => currentDocuments.Union(documents),
        _ => []
    };
}

async Task CreateInvertedIndex()
{
    var fileNames = index.Select(x => x.Split(" - ")[0]);

    var invertedIndex = new SortedDictionary<string, HashSet<string>>();

    foreach (var fileName in fileNames)
    {
        var tokens = await File.ReadAllTextAsync($"../artifacts/tokens/{fileName}.txt");

        foreach (var token in tokens.Trim().Split("\r\n"))
        {
            invertedIndex.TryAdd(token, []);
            invertedIndex[token].Add(fileName);
        }
    }

    foreach (var kvp in invertedIndex)
    {
        var invertedIndexEntry = $"{kvp.Key} - {string.Join(", ", kvp.Value)}\n";
        await File.AppendAllTextAsync("../artifacts/inverted-index.txt", invertedIndexEntry);
    }
}