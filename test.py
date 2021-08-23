import asyncio
import rqpy


proxies = {"all": "socks5://127.0.0.1:7890"}
headers = {'range': 'bytes=0-1023'}
json = {'usr': 'hello', 'wu': {'n': 9090}}

async def get(client, url):
    filename = url.split('/')[-1]
    resp = await client.request('GET', url)
    with open(filename, 'wb') as fp:
        return await resp.write_bytes(fp)


async def bare_get(url):
    return await rqpy.get(url, json = json)

urls = [
    "https://img.zcool.cn/community/019b1a611f2ff711013f47206bdd0f.jpg",
    "https://img.zcool.cn/community/01c67a611f2ff811013f4720dd89ae.jpg",
    "https://img.zcool.cn/community/01fdab611f2ff911013eaf702ae13e.jpg",
    "https://img.zcool.cn/community/019ecf611f2ffd11013eaf705936bd.jpg",
    "https://img.zcool.cn/community/01c943611f300a11013f4720e71709.jpg",
    "https://img.zcool.cn/community/013fda611f2ffb11013eaf70f4dfee.jpg",
    "https://img.zcool.cn/community/01dc1c611f2ffe11013f4720413bb4.jpg",
    "https://img.zcool.cn/community/011813611f300111013f4720010f93.jpg",
]

client = rqpy.RSClient(proxies=proxies, headers = headers)
tasks = asyncio.wait([get(client, url) for url in urls])

loop = asyncio.get_event_loop()
print(loop.run_until_complete(tasks))
resp = loop.run_until_complete(bare_get(urls[0]))
print(resp.content_length())
print(loop.run_until_complete(bare_get("http://127.0.0.1:8080")))

