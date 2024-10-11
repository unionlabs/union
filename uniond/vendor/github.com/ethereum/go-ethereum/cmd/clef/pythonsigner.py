import sys
import subprocess

from tinyrpc.transports import ServerTransport
from tinyrpc.protocols.jsonrpc import JSONRPCProtocol
from tinyrpc.dispatch import public, RPCDispatcher
from tinyrpc.server import RPCServer

"""
This is a POC example of how to write a custom UI for Clef.
The UI starts the clef process with the '--stdio-ui' option
and communicates with clef using standard input / output.

The standard input/output is a relatively secure way to communicate,
as it does not require opening any ports or IPC files. Needless to say,
it does not protect against memory inspection mechanisms
where an attacker can access process memory.

To make this work install all the requirements:

  pip install -r requirements.txt
"""

try:
    import urllib.parse as urlparse
except ImportError:
    import urllib as urlparse


class StdIOTransport(ServerTransport):
    """Uses std input/output for RPC"""

    def receive_message(self):
        return None, urlparse.unquote(sys.stdin.readline())

    def send_reply(self, context, reply):
        print(reply)


class PipeTransport(ServerTransport):
    """Uses std a pipe for RPC"""

    def __init__(self, input, output):
        self.input = input
        self.output = output

    def receive_message(self):
        data = self.input.readline()
        print(">> {}".format(data))
        return None, urlparse.unquote(data)

    def send_reply(self, context, reply):
        reply = str(reply, "utf-8")
        print("<< {}".format(reply))
        self.output.write("{}\n".format(reply))


def sanitize(txt, limit=100):
    return txt[:limit].encode("unicode_escape").decode("utf-8")


def metaString(meta):
    """
    "meta":{"remote":"clef binary","local":"main","scheme":"in-proc","User-Agent":"","Origin":""}
    """  # noqa: E501
    message = (
        "\tRequest context:\n"
        "\t\t{remote} -> {scheme} -> {local}\n"
        "\tAdditional HTTP header data, provided by the external caller:\n"
        "\t\tUser-Agent: {user_agent}\n"
        "\t\tOrigin: {origin}\n"
    )
    return message.format(
        remote=meta.get("remote", "<missing>"),
        scheme=meta.get("scheme", "<missing>"),
        local=meta.get("local", "<missing>"),
        user_agent=sanitize(meta.get("User-Agent"), 200),
        origin=sanitize(meta.get("Origin"), 100),
    )


class StdIOHandler:
    def __init__(self):
        pass

    @public
    def approveTx(self, req):
        """
        Example request:

        {"jsonrpc":"2.0","id":20,"method":"ui_approveTx","params":[{"transaction":{"from":"0xDEADbEeF000000000000000000000000DeaDbeEf","to":"0xDEADbEeF000000000000000000000000DeaDbeEf","gas":"0x3e8","gasPrice":"0x5","maxFeePerGas":null,"maxPriorityFeePerGas":null,"value":"0x6","nonce":"0x1","data":"0x"},"call_info":null,"meta":{"remote":"clef binary","local":"main","scheme":"in-proc","User-Agent":"","Origin":""}}]}

        :param transaction: transaction info
        :param call_info: info abou the call, e.g. if ABI info could not be
        :param meta: metadata about the request, e.g. where the call comes from
        :return:
        """  # noqa: E501
        message = (
            "Sign transaction request:\n"
            "\t{meta_string}\n"
            "\n"
            "\tFrom: {from_}\n"
            "\tTo: {to}\n"
            "\n"
            "\tAuto-rejecting request"
        )
        meta = req.get("meta", {})
        transaction = req.get("transaction")
        sys.stdout.write(
            message.format(
                meta_string=metaString(meta),
                from_=transaction.get("from", "<missing>"),
                to=transaction.get("to", "<missing>"),
            )
        )
        return {
            "approved": False,
        }

    @public
    def approveSignData(self, req):
        """
        Example request:

        {"jsonrpc":"2.0","id":8,"method":"ui_approveSignData","params":[{"content_type":"application/x-clique-header","address":"0x0011223344556677889900112233445566778899","raw_data":"+QIRoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAlAAAAAAAAAAAAAAAAAAAAAAAAAAAoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAuQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIIFOYIFOYIFOoIFOoIFOppFeHRyYSBkYXRhIEV4dHJhIGRhdGEgRXh0cqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIgAAAAAAAAAAA==","messages":[{"name":"Clique header","value":"clique header 1337 [0x44381ab449d77774874aca34634cb53bc21bd22aef2d3d4cf40e51176cb585ec]","type":"clique"}],"call_info":null,"hash":"0xa47ab61438a12a06c81420e308c2b7aae44e9cd837a5df70dd021421c0f58643","meta":{"remote":"clef binary","local":"main","scheme":"in-proc","User-Agent":"","Origin":""}}]}
        """  # noqa: E501
        message = (
            "Sign data request:\n"
            "\t{meta_string}\n"
            "\n"
            "\tContent-type: {content_type}\n"
            "\tAddress: {address}\n"
            "\tHash: {hash_}\n"
            "\n"
            "\tAuto-rejecting request\n"
        )
        meta = req.get("meta", {})
        sys.stdout.write(
            message.format(
                meta_string=metaString(meta),
                content_type=req.get("content_type"),
                address=req.get("address"),
                hash_=req.get("hash"),
            )
        )

        return {
            "approved": False,
            "password": None,
        }

    @public
    def approveNewAccount(self, req):
        """
        Example request:

        {"jsonrpc":"2.0","id":25,"method":"ui_approveNewAccount","params":[{"meta":{"remote":"clef binary","local":"main","scheme":"in-proc","User-Agent":"","Origin":""}}]}
        """  # noqa: E501
        message = (
            "Create new account request:\n"
            "\t{meta_string}\n"
            "\n"
            "\tAuto-rejecting request\n"
        )
        meta = req.get("meta", {})
        sys.stdout.write(message.format(meta_string=metaString(meta)))
        return {
            "approved": False,
        }

    @public
    def showError(self, req):
        """
        Example request:

        {"jsonrpc":"2.0","method":"ui_showError","params":[{"text":"If you see this message, enter 'yes' to the next question"}]}

        :param message: to display
        :return:nothing
        """  # noqa: E501
        message = (
            "## Error\n{text}\n"
            "Press enter to continue\n"
        )
        text = req.get("text")
        sys.stdout.write(message.format(text=text))
        input()
        return

    @public
    def showInfo(self, req):
        """
        Example request:

        {"jsonrpc":"2.0","method":"ui_showInfo","params":[{"text":"If you see this message, enter 'yes' to next question"}]}

        :param message: to display
        :return:nothing
        """  # noqa: E501
        message = (
            "## Info\n{text}\n"
            "Press enter to continue\n"
        )
        text = req.get("text")
        sys.stdout.write(message.format(text=text))
        input()
        return

    @public
    def onSignerStartup(self, req):
        """
        Example request:

        {"jsonrpc":"2.0", "method":"ui_onSignerStartup", "params":[{"info":{"extapi_http":"n/a","extapi_ipc":"/home/user/.clef/clef.ipc","extapi_version":"6.1.0","intapi_version":"7.0.1"}}]}
        """  # noqa: E501
        message = (
            "\n"
            "\t\tExt api url: {extapi_http}\n"
            "\t\tInt api ipc: {extapi_ipc}\n"
            "\t\tExt api ver: {extapi_version}\n"
            "\t\tInt api ver: {intapi_version}\n"
        )
        info = req.get("info")
        sys.stdout.write(
            message.format(
                extapi_http=info.get("extapi_http"),
                extapi_ipc=info.get("extapi_ipc"),
                extapi_version=info.get("extapi_version"),
                intapi_version=info.get("intapi_version"),
            )
        )

    @public
    def approveListing(self, req):
        """
        Example request:

        {"jsonrpc":"2.0","id":23,"method":"ui_approveListing","params":[{"accounts":[{"address":...
        """  # noqa: E501
        message = (
            "\n"
            "## Account listing request\n"
            "\t{meta_string}\n"
            "\tDo you want to allow listing the following accounts?\n"
            "\t-{addrs}\n"
            "\n"
            "->Auto-answering No\n"
        )
        meta = req.get("meta", {})
        accounts = req.get("accounts", [])
        addrs = [x.get("address") for x in accounts]
        sys.stdout.write(
            message.format(
                addrs="\n\t-".join(addrs),
                meta_string=metaString(meta)
            )
        )
        return {}

    @public
    def onInputRequired(self, req):
        """
        Example request:

        {"jsonrpc":"2.0","id":1,"method":"ui_onInputRequired","params":[{"title":"Master Password","prompt":"Please enter the password to decrypt the master seed","isPassword":true}]}

        :param message: to display
        :return:nothing
        """  # noqa: E501
        message = (
            "\n"
            "## {title}\n"
            "\t{prompt}\n"
            "\n"
            "> "
        )
        sys.stdout.write(
            message.format(
                title=req.get("title"),
                prompt=req.get("prompt")
            )
        )
        isPassword = req.get("isPassword")
        if not isPassword:
            return {"text": input()}

        return ""


def main(args):
    cmd = ["clef", "--stdio-ui"]
    if len(args) > 0 and args[0] == "test":
        cmd.extend(["--stdio-ui-test"])
    print("cmd: {}".format(" ".join(cmd)))

    dispatcher = RPCDispatcher()
    dispatcher.register_instance(StdIOHandler(), "ui_")

    # line buffered
    p = subprocess.Popen(
        cmd,
        bufsize=1,
        universal_newlines=True,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
    )

    rpc_server = RPCServer(
        PipeTransport(p.stdout, p.stdin), JSONRPCProtocol(), dispatcher
    )
    rpc_server.serve_forever()


if __name__ == "__main__":
    main(sys.argv[1:])
