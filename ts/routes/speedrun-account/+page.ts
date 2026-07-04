// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import type { PageLoad } from "./$types";
import { hostSyncStatus } from "./lib";

export const load = (async () => {
    return { status: await hostSyncStatus() };
}) satisfies PageLoad;
