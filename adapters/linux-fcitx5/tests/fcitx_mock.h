/* fcitx_mock.h — Lightweight mock of Fcitx5 Core API for standalone testing
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_FCITX_MOCK_H
#define TEXTVN_FCITX_MOCK_H

#include <cstdint>
#include <string>
#include <vector>
#include <bitset>
#include <functional>
#include <fcitx-utils/keysym.h>

namespace fcitx {

enum class KeyModifier : uint32_t {
    None = 0,
    Shift = 1 << 0,
    Ctrl = 1 << 1,
    Alt = 1 << 2,
    Super = 1 << 3,
};

enum class KeyState : uint32_t {
    None = 0,
    Shift = 1 << 0,
    CapsLock = 1 << 1,
    Ctrl = 1 << 2,
    Alt = 1 << 3,
    Super = 1 << 4,
};

class KeyStates {
public:
    KeyStates() : mask_(0) {}
    KeyStates(uint32_t m) : mask_(m) {}
    bool test(KeyState s) const { return (mask_ & static_cast<uint32_t>(s)) != 0; }
    void set(KeyState s) { mask_ |= static_cast<uint32_t>(s); }
private:
    uint32_t mask_;
};

class Key {
public:
    Key(uint32_t sym = 0, uint32_t mods = 0, bool isRelease = false)
        : sym_(sym), mods_(mods), release_(isRelease) {}

    uint32_t sym() const { return sym_; }
    bool isRelease() const { return release_; }
    bool check(uint32_t s) const { return sym_ == s; }

    bool hasModifier(KeyModifier m) const {
        return (mods_ & static_cast<uint32_t>(m)) != 0;
    }

    KeyStates states() const {
        uint32_t st = 0;
        if (hasModifier(KeyModifier::Shift)) st |= static_cast<uint32_t>(KeyState::Shift);
        if (hasModifier(KeyModifier::Ctrl))  st |= static_cast<uint32_t>(KeyState::Ctrl);
        if (hasModifier(KeyModifier::Alt))   st |= static_cast<uint32_t>(KeyState::Alt);
        if (hasModifier(KeyModifier::Super)) st |= static_cast<uint32_t>(KeyState::Super);
        return KeyStates(st);
    }

    static uint32_t keySymToUnicode(uint32_t sym) {
        if (sym >= 0x20 && sym <= 0x7E) return sym;
        return 0;
    }

private:
    uint32_t sym_;
    uint32_t mods_;
    bool     release_;
};

enum class CapabilityFlag : uint64_t {
    None = 0,
    SurroundingText = 1 << 0,
    Preedit = 1 << 1,
};

class CapabilityFlags {
public:
    CapabilityFlags() : flags_(0) {}
    void set(CapabilityFlag flag) { flags_ |= static_cast<uint64_t>(flag); }
    bool test(CapabilityFlag flag) const { return (flags_ & static_cast<uint64_t>(flag)) != 0; }
private:
    uint64_t flags_;
};

enum class TextFormatFlag : uint32_t {
    None = 0,
    Underline = 1 << 0,
};

class Text {
public:
    Text() = default;
    Text(const std::string &s) : text_(s) {}
    void append(const std::string &s, TextFormatFlag flag) {
        text_ += s;
        has_underline_ = (flag == TextFormatFlag::Underline);
    }
    const std::string &toString() const { return text_; }
    bool hasUnderline() const { return has_underline_; }

private:
    std::string text_;
    bool has_underline_ = false;
};

class InputPanel {
public:
    void reset() {
        preedit_.clear();
        has_underline_ = false;
    }
    void setClientPreedit(const Text &t) {
        preedit_ = t.toString();
        has_underline_ = t.hasUnderline();
    }
    const std::string &preedit() const { return preedit_; }
    bool hasUnderline() const { return has_underline_; }

private:
    std::string preedit_;
    bool has_underline_ = false;
};

class InputContext {
public:
    InputContext(const std::string &program = "test-app")
        : program_(program) {}

    const std::string &program() const { return program_; }
    CapabilityFlags &capabilityFlags() { return caps_; }
    const CapabilityFlags &capabilityFlags() const { return caps_; }
    InputPanel &inputPanel() { return panel_; }
    void updatePreedit() { preedit_updated_ = true; }

    void commitString(const std::string &str) {
        committed_text_ += str;
        last_commit_ = str;
    }

    void deleteSurroundingText(int offset, unsigned int size) {
        deleted_offsets_.push_back(offset);
        deleted_sizes_.push_back(size);
    }

    void forwardKey(const Key &k) {
        forwarded_keys_.push_back(k.sym());
    }

    /* Test state inspection */
    const std::string &committedText() const { return committed_text_; }
    const std::string &lastCommit() const { return last_commit_; }
    const std::vector<int> &deletedOffsets() const { return deleted_offsets_; }
    const std::vector<unsigned int> &deletedSizes() const { return deleted_sizes_; }
    const std::vector<uint32_t> &forwardedKeys() const { return forwarded_keys_; }

    struct Destroyed {};

    template <typename SignalType, typename Callback>
    void connect(Callback &&cb) {
        destroyed_callbacks_.push_back(std::forward<Callback>(cb));
    }

    void destroy() {
        for (auto &cb : destroyed_callbacks_) {
            cb();
        }
        destroyed_callbacks_.clear();
    }

    ~InputContext() {
        destroy();
    }

    void resetTestState() {
        committed_text_.clear();
        last_commit_.clear();
        deleted_offsets_.clear();
        deleted_sizes_.clear();
        forwarded_keys_.clear();
        preedit_updated_ = false;
        panel_.reset();
    }

private:
    std::string program_;
    CapabilityFlags caps_;
    InputPanel panel_;
    bool preedit_updated_ = false;

    std::string committed_text_;
    std::string last_commit_;
    std::vector<int> deleted_offsets_;
    std::vector<unsigned int> deleted_sizes_;
    std::vector<uint32_t> forwarded_keys_;
    std::vector<std::function<void()>> destroyed_callbacks_;
};

class KeyEvent {
public:
    KeyEvent(InputContext *ic, const Key &k)
        : ic_(ic), key_(k), filtered_(false) {}

    InputContext *inputContext() { return ic_; }
    const Key &key() const { return key_; }

    void filterAndAccept() { filtered_ = true; }
    bool isFiltered() const { return filtered_; }

private:
    InputContext *ic_;
    Key key_;
    bool filtered_;
};

class InputContextEvent {
public:
    InputContextEvent(InputContext *ic) : ic_(ic) {}
    InputContext *inputContext() { return ic_; }
private:
    InputContext *ic_;
};

class InputMethodEntry {};

class Instance {};

class AddonManager {
public:
    Instance *instance() { return &instance_; }
    void registerInputMethod(void *, const char *, const char *, const char *, const char *, const char *) {}
private:
    Instance instance_;
};

class AddonInstance {
public:
    virtual ~AddonInstance() = default;
};

class AddonFactory {
public:
    virtual ~AddonFactory() = default;
    virtual AddonInstance *create(AddonManager *manager) = 0;
};

class InputMethodEngineV2 {
public:
    InputMethodEngineV2(Instance *) {}
    virtual ~InputMethodEngineV2() = default;
    virtual void keyEvent(const InputMethodEntry &, KeyEvent &) {}
    virtual void reset(const InputMethodEntry &, InputContextEvent &) {}
    virtual void activate(const InputMethodEntry &, InputContextEvent &) {}
    virtual void deactivate(const InputMethodEntry &, InputContextEvent &) {}
};

} // namespace fcitx

#endif // TEXTVN_FCITX_MOCK_H
