import { invoke } from "@tauri-apps/api"
import type { Snapshot } from "../types/snapshot"

export async function set_profile(profile: string) { await invoke("set_profile", { profile }) }
export async function get_profile() { return await invoke("get_profile") as string }
export async function get_profiles() { return await invoke("get_profiles") as string[] }
export async function get_current() { return await invoke("get_current") as number | undefined }
export async function get_snapshots() { return await invoke("get_snapshots") as Snapshot[] }
export async function snapshot(snapshot: Snapshot) { await invoke("snapshot", { snapshot }) }
export async function restore(snapshot: Snapshot) { await invoke("restore", { snapshot }) }
export async function clear(name: string) { invoke("clear", { name }) }
