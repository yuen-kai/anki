// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PageLoad } from "./$types";
import { aiStatus } from "./lib";

export const load = (async () => {
    // In a plain browser there is no host backend; the screen still renders and
    // explains what is missing rather than failing.
    let available = false;
    let hostAvailable = true;
    try {
        available = (await aiStatus()).available;
    } catch {
        hostAvailable = false;
    }
    return { available, hostAvailable };
}) satisfies PageLoad;
