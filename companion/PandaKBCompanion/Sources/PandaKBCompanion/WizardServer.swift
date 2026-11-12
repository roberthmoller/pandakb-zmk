import Foundation

/// Local Vite/SvelteKit process for the bundled ZmkWiz UI.
final class WizardServer {
    static let shared = WizardServer()

    let port: UInt16 = 17_301
    private(set) var lastError: String?

    private var process: Process?
    private let queue = DispatchQueue(label: "PandaKB.WizardServer")

    var baseURL: URL {
        URL(string: "http://127.0.0.1:\(port)")!
    }

    var isRunning: Bool {
        process?.isRunning == true
    }

    func ensureRunning(completion: @escaping (Result<URL, Error>) -> Void) {
        queue.async {
            do {
                if self.isRunning {
                    try self.waitUntilReady(timeout: 2)
                    DispatchQueue.main.async { completion(.success(self.baseURL)) }
                    return
                }

                let root = try Self.resolveWizardRoot()
                try self.ensureDependencies(at: root)
                try self.startProcess(at: root)
                try self.waitUntilReady(timeout: 45)
                self.lastError = nil
                DispatchQueue.main.async { completion(.success(self.baseURL)) }
            } catch {
                self.lastError = error.localizedDescription
                DispatchQueue.main.async { completion(.failure(error)) }
            }
        }
    }

    func stop() {
        queue.sync {
            guard let process, process.isRunning else {
                self.process = nil
                return
            }
            process.terminate()
            process.waitUntilExit()
            self.process = nil
        }
    }

    // MARK: - Paths

    static func resolveWizardRoot() throws -> URL {
        if let env = ProcessInfo.processInfo.environment["PANDAKB_ZMKWIZ"], !env.isEmpty {
            let url = URL(fileURLWithPath: env, isDirectory: true)
            if packageJSONExists(at: url) { return url }
            throw WizardError.missingWizard("PANDAKB_ZMKWIZ set but no package.json at \(env)")
        }

        // Compile-time path: …/companion/PandaKBCompanion/Sources/PandaKBCompanion/WizardServer.swift
        let fromSource = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("zmkwiz", isDirectory: true)
        if packageJSONExists(at: fromSource) { return fromSource }

        var dir = URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
        for _ in 0..<8 {
            let direct = dir.appendingPathComponent("zmkwiz", isDirectory: true)
            if packageJSONExists(at: direct) { return direct }
            let nested = dir.appendingPathComponent("companion/zmkwiz", isDirectory: true)
            if packageJSONExists(at: nested) { return nested }
            let parent = dir.deletingLastPathComponent()
            if parent.path == dir.path { break }
            dir = parent
        }

        throw WizardError.missingWizard(
            "Could not find companion/zmkwiz. Set PANDAKB_ZMKWIZ or run from the repo."
        )
    }

    private static func packageJSONExists(at url: URL) -> Bool {
        FileManager.default.fileExists(atPath: url.appendingPathComponent("package.json").path)
    }

    // MARK: - Process

    private func ensureDependencies(at root: URL) throws {
        let nodeModules = root.appendingPathComponent("node_modules")
        if FileManager.default.fileExists(atPath: nodeModules.path) { return }

        let install = Process()
        install.currentDirectoryURL = root
        install.executableURL = URL(fileURLWithPath: "/bin/zsh")
        install.arguments = [
            "-lc",
            Self.nvmPrefix + "npm run install:deps",
        ]
        let pipe = Pipe()
        install.standardOutput = pipe
        install.standardError = pipe
        try install.run()
        install.waitUntilExit()
        if install.terminationStatus != 0 {
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let text = String(data: data, encoding: .utf8) ?? ""
            throw WizardError.installFailed(text.trimmingCharacters(in: .whitespacesAndNewlines))
        }
    }

    private func startProcess(at root: URL) throws {
        let process = Process()
        process.currentDirectoryURL = root
        process.executableURL = URL(fileURLWithPath: "/bin/zsh")
        process.arguments = [
            "-lc",
            Self.nvmPrefix
                + "npm run dev -- --host 127.0.0.1 --port \(port) --strictPort",
        ]
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe
        process.terminationHandler = { [weak self] _ in
            self?.queue.async {
                if self?.process === process {
                    self?.process = nil
                }
            }
        }
        try process.run()
        self.process = process
    }

    private func waitUntilReady(timeout: TimeInterval) throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if process?.isRunning != true {
                throw WizardError.serverExited(lastError ?? "wizard process exited")
            }
            if isHTTPReady() { return }
            Thread.sleep(forTimeInterval: 0.25)
        }
        throw WizardError.timeout("ZmkWiz did not become ready on port \(port)")
    }

    /// Load nvm and honor companion/zmkwiz `.nvmrc` (needs Node ≥ 22.17).
    private static let nvmPrefix =
        #"export NVM_DIR="$HOME/.nvm"; [ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"; nvm use >/dev/null; "#

    private func isHTTPReady() -> Bool {
        var request = URLRequest(url: baseURL)
        request.timeoutInterval = 0.5
        let sem = DispatchSemaphore(value: 0)
        var ok = false
        URLSession.shared.dataTask(with: request) { _, response, _ in
            if let http = response as? HTTPURLResponse, (200..<500).contains(http.statusCode) {
                ok = true
            }
            sem.signal()
        }.resume()
        _ = sem.wait(timeout: .now() + 0.6)
        return ok
    }
}

enum WizardError: LocalizedError {
    case missingWizard(String)
    case installFailed(String)
    case serverExited(String)
    case timeout(String)

    var errorDescription: String? {
        switch self {
        case .missingWizard(let message),
             .installFailed(let message),
             .serverExited(let message),
             .timeout(let message):
            return message
        }
    }
}
