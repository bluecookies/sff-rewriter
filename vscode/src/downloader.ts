import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import { Readable } from "stream";

const GITHUB_REPO = "bluecookies/sff-rewriter";

const platformBinary = (): string => {
  switch (`${os.platform()}-${os.arch()}`) {
    case "darwin-arm64":
      return "sff-fmt-darwin-arm64";
    case "win32-x64":
      return "sff-fmt-windows-x64.exe";
    default:
      throw new Error(`Unsupported platform: ${os.platform()}-${os.arch()}`);
  }
};

export const ensureBinary = async (
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel,
): Promise<string> => {
  // Check user-configured path first
  const configPath = vscode.workspace
    .getConfiguration("sff")
    .get<string>("binaryPath");
  if (configPath) {
    return configPath;
  }

  // Check if already downloaded
  const binaryName = platformBinary();
  const binaryPath = path.join(context.globalStorageUri.fsPath, binaryName);
  if (fs.existsSync(binaryPath)) {
    return binaryPath;
  }

  // Download from GitHub releases
  const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/latest/download/${binaryName}`;
  await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "Downloading sff-rewriter...",
    },
    async () => {
      const url = downloadUrl;
      await download(url, binaryPath, output);
      if (os.platform() !== "win32") {
        fs.chmodSync(binaryPath, 0o755);
      }
    },
  );

  return binaryPath;
};

const download = async (
  url: string,
  dest: string,
  output: vscode.OutputChannel,
): Promise<void> => {
  fs.mkdirSync(path.dirname(dest), { recursive: true });

  output.appendLine(`Fetching binary from ${url}`);

  let res: Response;
  try {
    res = await fetch(url);
  } catch (err: any) {
    output.appendLine(`Network error: ${err.message}`);
    throw err;
  }

  output.appendLine(`Status: ${res.status} ${res.statusText}`);

  if (!res.ok) {
    throw new Error(`Download failed: ${res.status} ${res.statusText}`);
  }

  if (res.redirected) {
    output.appendLine(`Redirected to ${res.url}`);
  }

  const file = fs.createWriteStream(dest);

  try {
    await new Promise<void>((resolve, reject) => {
      const nodeStream = Readable.fromWeb(res.body as any);

      nodeStream.pipe(file);

      nodeStream.on("error", reject);
      file.on("error", reject);
      file.on("finish", resolve);
    });

    output.appendLine(`Saved to ${dest}`);
  } catch (err: any) {
    output.appendLine(`Stream/write error: ${err.message}`);

    // Clean up partial file
    try {
      fs.unlinkSync(dest);
      output.appendLine(`Deleted partial file`);
    } catch {}

    throw err;
  }

  output.appendLine(`Saved to ${dest}`);
};
