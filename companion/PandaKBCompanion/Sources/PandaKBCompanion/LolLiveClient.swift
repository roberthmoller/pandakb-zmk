import Foundation

/// Polls the League of Legends Live Client Data API (local only while in-game).
/// https://127.0.0.1:2999/liveclientdata/allgamedata
final class LolLiveClient: ObservableObject {
    @Published private(set) var status: GameStatus?
    @Published private(set) var inGame = false

    private var timer: Timer?
    private let session: URLSession
    private let url = URL(string: "https://127.0.0.1:2999/liveclientdata/allgamedata")!

    init() {
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = 1.5
        config.timeoutIntervalForResource = 1.5
        config.waitsForConnectivity = false
        session = URLSession(configuration: config, delegate: InsecureLocalhostDelegate.shared, delegateQueue: nil)
    }

    func start() {
        stop()
        let t = Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { [weak self] _ in
            self?.poll()
        }
        RunLoop.main.add(t, forMode: .common)
        timer = t
        poll()
    }

    func stop() {
        timer?.invalidate()
        timer = nil
    }

    private func poll() {
        session.dataTask(with: url) { [weak self] data, response, error in
            guard let self else { return }
            DispatchQueue.main.async {
                self.handle(data: data, response: response, error: error)
            }
        }.resume()
    }

    private func handle(data: Data?, response: URLResponse?, error: Error?) {
        guard error == nil,
              let http = response as? HTTPURLResponse,
              http.statusCode == 200,
              let data,
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else {
            inGame = false
            status = nil
            return
        }

        inGame = true

        let activeName = ((json["activePlayer"] as? [String: Any])?["summonerName"] as? String)
            ?? ((json["activePlayer"] as? [String: Any])?["riotIdGameName"] as? String)

        var kills: UInt8 = 0
        var deaths: UInt8 = 0
        var assists: UInt8 = 0
        var cs: UInt16 = 0
        var level: UInt8 = 0

        if let active = json["activePlayer"] as? [String: Any] {
            if let lvl = active["level"] as? Int {
                level = UInt8(clamping: lvl)
            }
        }

        if let players = json["allPlayers"] as? [[String: Any]] {
            let me = players.first { player in
                if let activeName, !activeName.isEmpty {
                    let summoner = player["summonerName"] as? String
                    let riot = player["riotIdGameName"] as? String
                    return summoner == activeName || riot == activeName
                }
                return (player["isBot"] as? Bool) != true
            } ?? players.first

            if let scores = me?["scores"] as? [String: Any] {
                kills = UInt8(clamping: (scores["kills"] as? Int) ?? 0)
                deaths = UInt8(clamping: (scores["deaths"] as? Int) ?? 0)
                assists = UInt8(clamping: (scores["assists"] as? Int) ?? 0)
                cs = UInt16(clamping: (scores["creepScore"] as? Int) ?? 0)
            }
            if level == 0, let lvl = me?["level"] as? Int {
                level = UInt8(clamping: lvl)
            }
        }

        status = GameStatus(
            badge: .lol,
            kills: kills,
            deaths: deaths,
            assists: assists,
            cs: cs,
            level: level,
            statsValid: true
        )
    }
}

/// Live Client API uses a self-signed cert on localhost.
private final class InsecureLocalhostDelegate: NSObject, URLSessionDelegate {
    static let shared = InsecureLocalhostDelegate()

    func urlSession(
        _ session: URLSession,
        didReceive challenge: URLAuthenticationChallenge,
        completionHandler: @escaping (URLSession.AuthChallengeDisposition, URLCredential?) -> Void
    ) {
        if challenge.protectionSpace.host == "127.0.0.1" || challenge.protectionSpace.host == "localhost",
           let trust = challenge.protectionSpace.serverTrust {
            completionHandler(.useCredential, URLCredential(trust: trust))
        } else {
            completionHandler(.performDefaultHandling, nil)
        }
    }
}
