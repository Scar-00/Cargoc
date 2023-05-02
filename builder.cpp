#include "builder.h"
#include <corecrt_wstdio.h>
#include <cstdlib>
#include <stdio.h>
#include <stdlib.h>

CmdBuilder::CmdBuilder(const char *command) {
    this->cmds.push_back(command);
}

CmdBuilder &CmdBuilder::Arg(const char *arg) {
    this->cmds.back().args.push_back(arg);
    return *this;
}

CmdBuilder &CmdBuilder::Add(const char *command) {
    this->cmds.push_back(command);
    return *this;
}

CmdBuilder &CmdBuilder::Seperate(bool seperate) {
    this->seperate = seperate;
    return *this;
}

void CmdBuilder::Execute() {
    std::string command;
    for(auto &cmd : cmds) {
        if(seperate) {
            std::string command = cmd.ConstructCommand();
            printf("[Executing] -> %s\n", command.c_str());
            fflush(stdout);
            system(command.c_str());
        }else {
            command.append(cmd.ConstructCommand());
            command.append("&&");
        }
    }
    if(seperate) return;
    printf("[Executing] -> %s\n", command.c_str());
    system(command.c_str());
}

std::string CmdBuilder::Command::ConstructCommand() {
    std::string cmd = this->cmd;
    for(auto &arg : this->args) {
        cmd.append(arg);
    }
    return cmd;
}
