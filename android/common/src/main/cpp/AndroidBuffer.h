#pragma once

#include <streambuf>

class AndroidBuffer : public std::streambuf
{
public:
	static constexpr auto BUFFER_SIZE = 512;

	AndroidBuffer() { setp(buffer, buffer + BUFFER_SIZE - 1); }

private:
	int overflow(int c)
	{
		if (c == traits_type::eof())
		{
			*pptr() = traits_type::to_char_type(c);
			sbumpc();
		}
		return sync() ? traits_type::eof() : traits_type::not_eof(c);
	}

	int sync()
	{
		int rc = 0;
		if (pbase() != pptr())
		{
			char writeBuffer[BUFFER_SIZE + 1]{};
			memcpy(writeBuffer, pbase(), pptr() - pbase());

			rc = __android_log_write(ANDROID_LOG_VERBOSE, "std", writeBuffer) > 0;
			setp(buffer, buffer + BUFFER_SIZE - 1);
		}
		return rc;
	}

	char buffer[BUFFER_SIZE];
};
