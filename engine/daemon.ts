import { loginWithQR, loginWithAuthToken } from "../linejs/packages/linejs/client/login.ts";
import { FileStorage } from "../linejs/packages/linejs/base/storage/file.ts";
import { Client } from "../linejs/packages/linejs/client/client.ts";

const CRED_PATH = "./line_credentials.json";
const STORAGE_PATH = "./line_storage.json";

const storage = new FileStorage(STORAGE_PATH);

async function renderTerminalQR(url: string) {
	try {
		const cmd = new Deno.Command("qrencode", {
			args: ["-t", "ANSIUTF8", url],
			stdout: "piped",
		});
		const output = await cmd.output();
		console.log(new TextDecoder().decode(output.stdout));
	} catch (e) {
		console.log(`(Failed to render ASCII QR: ${e})`);
	}
}

export async function getClient(): Promise<Client> {
	// Try loading saved credentials
	try {
		if (await Deno.stat(CRED_PATH).then(() => true).catch(() => false)) {
			const credRaw = await Deno.readTextFile(CRED_PATH);
			const cred = JSON.parse(credRaw);
			if (cred.accessToken) {
				console.log("🔄 Loading saved LINE session...");
				const client = await loginWithAuthToken(cred, {
					device: "DESKTOPWIN",
					storage,
				});
				await client.getMyProfile();
				console.log("✅ Restored session successfully!");
				return client;
			}
		}
	} catch (e) {
		console.log(`(Session restore failed: ${e}, requesting new login)`);
	}

	console.log("\n📱 Generating LINE QR Code Login...\n");

	const client = await loginWithQR(
		{
			async onReceiveQRUrl(url: string) {
				console.log("==================================================");
				console.log("👉 Please scan this QR code with your LINE mobile app:");
				console.log("   (Open LINE -> Scan QR icon at top right)\n");
				await renderTerminalQR(url);
				console.log(`Raw URL: ${url}`);
				console.log("==================================================\n");
			},
			onPincodeRequest(pin: string) {
				console.log("\n**************************************************");
				console.log(`🔐 VERIFICATION PIN:    >>>   ${pin}   <<<`);
				console.log("👉 Enter this 4-digit PIN on your mobile phone screen!");
				console.log("**************************************************\n");
			},
		},
		{
			device: "DESKTOPWIN",
			storage,
		},
	);

	// Persist the active token
	const token = client.base.authToken;
	const refreshToken = (await storage.get("refreshToken")) as string | undefined;
	const expire = (await storage.get("expire")) as number | undefined;

	const credData = {
		accessToken: token,
		refreshToken: refreshToken,
		expire: expire,
	};

	await Deno.writeTextFile(CRED_PATH, JSON.stringify(credData, null, 2));
	console.log("💾 Credentials saved permanently to", CRED_PATH);
	return client;
}

if (import.meta.main) {
	console.log("🚀 Starting LINE Native Engine (Port 4884)...");
	const client = await getClient();
	const p = await client.getMyProfile();
	console.log("\n==================================================");
	console.log("🎉 CONNECTED TO REAL LINE ACCOUNT!");
	console.log("👤 Name   :", p.displayName);
	console.log("💬 Status :", p.statusMessage || "(none)");
	console.log("🆔 MID    :", p.mid);
	console.log("==================================================");

	// Start local API server on 127.0.0.1:4884 for the Rust Libadwaita GUI
	Deno.serve({ port: 4884, hostname: "127.0.0.1" }, async (req) => {
		const url = new URL(req.url);

		// Profile
		if (url.pathname === "/api/me") {
			const prof = await client.getMyProfile();
			return Response.json({
				displayName: prof.displayName,
				statusMessage: prof.statusMessage || "",
				mid: prof.mid,
				pictureUrl: prof.picturePath || "",
			});
		}

		// Joined chats
		if (url.pathname === "/api/chats") {
			try {
				const chats = await client.fetchJoinedChats();
				const items = chats.map((c) => ({
					mid: c.mid,
					name: c.name || "Direct Chat",
					lastMessage: "แตะเพื่อดูข้อความ",
					time: "",
				}));
				return Response.json(items);
			} catch (e) {
				return Response.json({ error: String(e) }, { status: 500 });
			}
		}

		// Send message
		if (url.pathname.startsWith("/api/chats/") && url.pathname.endsWith("/send") && req.method === "POST") {
			try {
				const body = await req.json();
				const parts = url.pathname.split("/");
				const chatMid = parts[3];
				const chat = await client.getChat(chatMid);
				await chat.sendMessage(body.text);
				return Response.json({ success: true });
			} catch (e) {
				return Response.json({ error: String(e) }, { status: 500 });
			}
		}

		return new Response("LINE Native Linux API Server", { status: 200 });
	});
}
