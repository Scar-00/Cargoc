#pragma once

#include <string>
#include <vector>

struct CmdBuilder {
    public:
    CmdBuilder(const char *command);
    CmdBuilder &Arg(const char *arg);
    CmdBuilder &Add(const char *command);
    CmdBuilder &Seperate(bool seperate);
    void Execute();
    private:
    struct Command {
        Command(const char *cmd): cmd(cmd) {}
        std::string ConstructCommand();
        std::string cmd;
        std::vector<std::string> args;
    };
    std::vector<Command> cmds;
    bool seperate = false;
};
