// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { expect, test } from "vitest";

import { type ExtractedFile, extractFile, openingMessage, sourcesForRequest } from "./lib";

// A minimal File stand-in: extractFile only reads `.name` and `.text()` on the
// plain-text path, so this exercises extraction without a DOM File.
function textFile(name: string, text: string): File {
    return { name, text: async () => text } as unknown as File;
}

function failingFile(name: string): File {
    return {
        name,
        text: async () => {
            throw new Error("unreadable");
        },
    } as unknown as File;
}

test("openingMessage uses the note, or a plain default when blank", () => {
    expect(openingMessage("  cover chapters 3 and 4  ")).toBe("cover chapters 3 and 4");
    expect(openingMessage("   ")).toBe("Turn the source material into a study deck.");
    expect(openingMessage("")).toBe("Turn the source material into a study deck.");
});

test("sourcesForRequest drops empty files and keeps name + text", () => {
    const files: ExtractedFile[] = [
        { name: "a.txt", text: "real content", truncated: false },
        { name: "b.txt", text: "   ", truncated: false },
        { name: "c.pdf", text: "", truncated: false, error: "could not read" },
    ];
    expect(sourcesForRequest(files)).toEqual([{ name: "a.txt", text: "real content" }]);
});

test("extractFile reads plain text files", async () => {
    const result = await extractFile(textFile("notes.md", "# Enzymes\nlower activation energy"));
    expect(result.name).toBe("notes.md");
    expect(result.text).toContain("Enzymes");
    expect(result.truncated).toBe(false);
    expect(result.error).toBeUndefined();
});

test("extractFile caps very long files and flags truncation", async () => {
    const result = await extractFile(textFile("big.csv", "x".repeat(100_001)));
    expect(result.truncated).toBe(true);
    expect(result.text.length).toBe(100_000);
});

test("extractFile degrades to an error entry instead of throwing", async () => {
    const result = await extractFile(failingFile("broken.txt"));
    expect(result.text).toBe("");
    expect(result.error).toBeTruthy();
});
