// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { goto } from "$app/navigation";
import { speedrunAiConfig, speedrunAiImport } from "@generated/backend";

import { dec, enc, type Hierarchy, quiet, saveHierarchy } from "../speedrun-hierarchy/lib";

// One chat turn in the clarification conversation.
export interface ChatMsg {
    role: "user" | "assistant";
    content: string;
}

// A file the user imported, after in-browser text extraction.
export interface ExtractedFile {
    name: string;
    text: string;
    // The text was cut to the client size cap.
    truncated: boolean;
    // Set when extraction failed; text is then empty.
    error?: string;
}

// Whether AI import is configured (a developer-supplied key is present).
export interface AiStatus {
    available: boolean;
}

export interface DeckCounts {
    groups: number;
    topics: number;
    concepts: number;
}

// The backend's `{ kind: ... }` result for one import turn.
export type ImportResponse =
    | { kind: "message"; content: string; ready: boolean }
    | { kind: "deck"; hierarchy: Hierarchy; summary: DeckCounts }
    | { kind: "error"; message: string; retryable: boolean };

interface ImportRequestBody {
    sources: { name: string; text: string }[];
    messages: ChatMsg[];
    phase: "clarify" | "generate";
    deckTitle?: string;
}

// --- backend RPCs -----------------------------------------------------------

/** Whether AI import is available; the key is developer-supplied server-side. */
export async function aiStatus(): Promise<AiStatus> {
    return dec<AiStatus>(await speedrunAiConfig({ json: enc({}) }, quiet));
}

/** Run one import turn (clarify or generate). */
export async function aiImport(body: ImportRequestBody): Promise<ImportResponse> {
    return dec<ImportResponse>(await speedrunAiImport({ json: enc(body) }, quiet));
}

// The extracted files carry UI-only fields (truncated/error); strip to the
// wire shape the backend expects.
export function sourcesForRequest(files: ExtractedFile[]): { name: string; text: string }[] {
    return files
        .filter((file) => file.text.trim().length > 0)
        .map((file) => ({ name: file.name, text: file.text }));
}

/** Save the reviewed deck and hand off to the builder for editing. */
export async function openInBuilder(hierarchy: Hierarchy): Promise<void> {
    const result = await saveHierarchy(hierarchy);
    await goto(`/speedrun-hierarchy/${result.deckId}`);
}

// --- in-browser text extraction --------------------------------------------

// Cap per file so a huge upload cannot bloat the request; the backend caps
// again for the prompt itself.
const CLIENT_MAX_CHARS = 100_000;

// pdf.js and fflate are pulled from a CDN only when a PDF or DOCX is actually
// dropped, so there is no heavy npm dependency and the rest of the app is
// unaffected when this screen is never used.
const PDFJS_VERSION = "4.7.76";
const FFLATE_VERSION = "0.8.2";

interface PdfTextItem {
    str?: string;
}
interface PdfTextContent {
    items: PdfTextItem[];
}
interface PdfPage {
    getTextContent(): Promise<PdfTextContent>;
}
interface PdfDoc {
    numPages: number;
    getPage(pageNumber: number): Promise<PdfPage>;
}
interface PdfjsModule {
    GlobalWorkerOptions: { workerSrc: string };
    getDocument(src: { data: Uint8Array }): { promise: Promise<PdfDoc> };
}
interface FflateModule {
    unzipSync(data: Uint8Array): Record<string, Uint8Array>;
}

function extensionOf(name: string): string {
    const dot = name.lastIndexOf(".");
    return dot >= 0 ? name.slice(dot + 1).toLowerCase() : "";
}

function capText(text: string): { text: string; truncated: boolean } {
    if (text.length <= CLIENT_MAX_CHARS) {
        return { text, truncated: false };
    }
    return { text: text.slice(0, CLIENT_MAX_CHARS), truncated: true };
}

async function readAsText(file: File): Promise<string> {
    return await file.text();
}

async function extractPdf(file: File): Promise<string> {
    const pdfjs = (await import(
        /* @vite-ignore */ `https://cdn.jsdelivr.net/npm/pdfjs-dist@${PDFJS_VERSION}/build/pdf.min.mjs`
    )) as unknown as PdfjsModule;
    pdfjs.GlobalWorkerOptions.workerSrc =
        `https://cdn.jsdelivr.net/npm/pdfjs-dist@${PDFJS_VERSION}/build/pdf.worker.min.mjs`;
    const data = new Uint8Array(await file.arrayBuffer());
    const doc = await pdfjs.getDocument({ data }).promise;
    const pages: string[] = [];
    for (let pageNumber = 1; pageNumber <= doc.numPages; pageNumber++) {
        const page = await doc.getPage(pageNumber);
        const content = await page.getTextContent();
        pages.push(content.items.map((item) => item.str ?? "").join(" "));
    }
    return pages.join("\n\n");
}

// A DOCX is a zip; the body text lives in word/document.xml. Unzip with fflate,
// turn paragraph/tab tags into whitespace, then strip the remaining tags.
async function extractDocx(file: File): Promise<string> {
    const { unzipSync } = (await import(
        /* @vite-ignore */ `https://cdn.jsdelivr.net/npm/fflate@${FFLATE_VERSION}/+esm`
    )) as unknown as FflateModule;
    const zip = unzipSync(new Uint8Array(await file.arrayBuffer()));
    const document = zip["word/document.xml"];
    if (!document) {
        throw new Error("no document body found");
    }
    const xml = new TextDecoder().decode(document);
    return xml
        .replace(/<\/w:p>/g, "\n")
        .replace(/<w:tab\b[^>]*\/>/g, "\t")
        .replace(/<[^>]+>/g, "")
        .replace(/&amp;/g, "&")
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">")
        .replace(/&quot;/g, "\"")
        .replace(/&#39;/g, "'")
        .replace(/\n{3,}/g, "\n\n")
        .trim();
}

/** Extract plain text from one file, degrading to an error entry on failure. */
export async function extractFile(file: File): Promise<ExtractedFile> {
    const ext = extensionOf(file.name);
    try {
        let raw: string;
        if (ext === "pdf") {
            raw = await extractPdf(file);
        } else if (ext === "docx") {
            raw = await extractDocx(file);
        } else {
            // txt, md, csv, and anything else we treat as plain text.
            raw = await readAsText(file);
        }
        const { text, truncated } = capText(raw);
        return { name: file.name, text, truncated };
    } catch (err) {
        const reason = err instanceof Error ? err.message : String(err);
        const hint = ext === "pdf" || ext === "docx"
            ? "Could not read this file. Check your connection, or paste the text into the note below."
            : "Could not read this file.";
        return { name: file.name, text: "", truncated: false, error: `${hint} (${reason})` };
    }
}

/** The user's opening message: their note, or a plain default. */
export function openingMessage(note: string): string {
    const trimmed = note.trim();
    if (trimmed) {
        return trimmed;
    }
    return "Turn the source material into a study deck.";
}
