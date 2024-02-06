from dosei import Dosei

port = 8080
dosei = Dosei(
    name="dosei-bot",
    port=port,
    command=f"ls",
    dev=f"ls"
)

@dosei.cron_job("* * * * *")
async def hello_world():
    print("hello xd")
